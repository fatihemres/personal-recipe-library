#!/usr/bin/env python3
"""Preparation only: never writes seed files, migrations, app profiles or approvals."""
import argparse
import csv
import difflib
import hashlib
import io
import json
import re
import unicodedata
import xml.etree.ElementTree as ET
import zipfile
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'catalog/preparation/m3d2a'


def read(name):
    return json.loads((OUT / name).read_text())


def encoded(value):
    return json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + '\n'


def normalize(text):
    return ' '.join(unicodedata.normalize('NFC', text).replace('I', 'ı').replace('İ', 'i').lower().split())


def check_baseline():
    for name, checksum in read('baseline-checksums.json').items():
        archived = name.replace('catalog/production/', 'catalog/releases/7/', 1) if name.startswith('catalog/production/') else 'catalog/releases/7/sources.json' if name == 'catalog/sources.json' else name
        assert hashlib.sha256((ROOT / archived).read_bytes()).hexdigest() == checksum, f'Protected file changed: {name}'


def extract(archive, ontology, selection):
    sources = {s['id']: s for s in read('sources.json')}
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == sources['usda']['archiveSha256']
    assert hashlib.sha256(ontology.read_bytes()).hexdigest() == sources['foodon']['artifactSha256']
    with zipfile.ZipFile(archive) as z:
        filename = next(n for n in z.namelist() if n.endswith('/food.csv'))
        foods = {r['fdc_id']: r for r in csv.DictReader(io.TextIOWrapper(z.open(filename)))}
    ns = {'o': 'http://www.w3.org/2002/07/owl#', 'r': 'http://www.w3.org/2000/01/rdf-schema#'}
    rdf = '{http://www.w3.org/1999/02/22-rdf-syntax-ns#}'
    terms = {c.get(rdf + 'about', '').rsplit('_', 1)[-1]: c for c in ET.parse(ontology).getroot().findall('o:Class', ns)}
    result = []
    for row in selection:
        key = row['id']
        if row['source'] == 'usda':
            result.append(dict(source='usda', id=key, label=foods[key]['description'], url=f'https://fdc.nal.usda.gov/food-details/{key}/nutrients'))
        else:
            term = terms[key]
            label = term.find('r:label', ns).text
            assert not label.startswith('obsolete')
            assert not any(x.tag.endswith('deprecated') and x.text == 'true' for x in term)
            lineage = [{'property': x.tag.rsplit('}', 1)[-1], 'value': x.text or x.get(rdf + 'resource', '')}
                       for x in term if any(t in x.tag for t in ['hasDbXref', 'IAO_0000119', 'isDefinedBy'])]
            result.append(dict(source='foodon', id=key, label=label, url=f'http://purl.obolibrary.org/obo/FOODON_{key}', lineage=lineage))
    return result


def words(text):
    # Similarity is a review signal, never proof of equivalence or authorization.
    skip = {'food', 'product', 'raw', 'fresh', 'dried', 'dry', 'cooked', 'ready', 'serve', 'whole', 'or', 'and', 'with', 'without', 'of', 'the', 'to', 'unsweetened', 'added', 'salt', 'canned'}
    return set(re.findall(r'[^\W\d_]+', normalize(text))) - skip


def collisions(row, evidence, catalog):
    hits = []
    for ingredient in catalog:
        labels = list(ingredient['names'].values()) + [a['name'] for a in ingredient['aliases']]
        source_match = row['source'] == 'usda' and any(p['sourceId'].startswith('usda') and p['externalId'] == row['id'] for p in ingredient['provenance'])
        exact = any(normalize(label) in {normalize(row['tr']), normalize(evidence['label'])} for label in labels)
        candidate_words = words(evidence['label'])
        similar = any(len(candidate_words & words(label)) >= 2 and len(candidate_words & words(label)) / max(len(candidate_words | words(label)), 1) >= .6 for label in labels)
        similar = similar or any(difflib.SequenceMatcher(None, normalize(row['tr']), normalize(label)).ratio() >= .86 for label in labels)
        if source_match or exact or similar:
            hits.append(dict(key=ingredient['key'], id=ingredient['id'], tr=ingredient['names']['tr'], en=ingredient['names']['en'],
                             kind='existing-source-identity' if source_match else 'exact-name-or-alias' if exact else 'possible-synonym-or-distinct-form'))
    return sorted(hits, key=lambda h: h['key'])


def build(selection, evidence, catalog, sources):
    by_evidence = {(e['source'], e['id']): e for e in evidence}
    assert len(by_evidence) == len(selection) == len(set((s['source'], s['id']) for s in selection))
    categories = {c['key'] for c in catalog['categories']}
    rows = []
    for item in selection:
        e = by_evidence[item['source'], item['id']]
        assert e['label'] and item['tr'] and set(item['categories'].split(',')) <= categories
        assert item['type'] in ('food', 'beverage') and item['confidence'] in ('high', 'medium')
        matches = collisions(item, e, catalog['ingredients'])
        existing = any(h['kind'] == 'existing-source-identity' for h in matches)
        status = ('reject-existing-identity' if existing else 'defer-source-clearance' if item['source'] != 'usda'
                  else 'review-identity-or-terminology' if matches or item['confidence'] != 'high' or item['concern'] else 'proposed-after-editorial-review')
        forms = [word for word in ('raw', 'fresh', 'dried', 'dry', 'powder', 'ground', 'canned', 'frozen', 'roasted', 'steeped', 'juice', 'syrup', 'paste', 'sauce', 'leaf', 'seed', 'bean', 'flour', 'cheese', 'oil') if word in e['label'].lower()]
        rows.append(dict(candidateId=f"m3d2a-{item['source']}-{item['id']}", source=item['source'], sourceIdentifier=item['id'],
                         sourceUrl=e['url'], sourceEnglishLabel=e['label'], proposedEnglishName=e['label'], proposedTurkishName=item['tr'],
                         proposedAliases=[], proposedCategories=item['categories'].split(','), ingredientType=item['type'],
                         form=', '.join(forms) or 'unspecified in label; do not infer', retainedSourceForm=e['label'],
                         licenseStatus=sources[item['source']]['decision'], collisions=matches,
                         collisionInterpretation='Similarity is not identity. Review preparation qualifiers and source lineage; never auto-merge.',
                         terminologyConfidence=item['confidence'] + '-editorial-not-human-approved', concern=item['concern'],
                         humanReviewRequired=True, humanDecision=None, reviewer=None, reviewDate=None,
                         proposedImportStatus=status, importAuthorized=False, observations=[]))
    return sorted(rows, key=lambda r: r['candidateId'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path)
    parser.add_argument('--ontology', type=Path)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    check_baseline()
    selection = list(csv.DictReader(io.StringIO((OUT / 'selection.tsv').read_text()), delimiter='\t'))
    evidence = read('source-records.json')
    if args.archive or args.ontology:
        assert args.archive and args.ontology, 'Supply both pinned artifacts'
        assert extract(args.archive, args.ontology, selection) == evidence, 'Source extract drift'
    catalog = json.loads((ROOT / 'catalog/releases/7/ingredients.json').read_text())
    sources = {s['id']: s for s in read('sources.json')}
    candidates = build(selection, evidence, catalog, sources)
    review = read('collision-review.json')
    keys = {i['key'] for i in catalog['ingredients']}
    for row in candidates:
        assessment = review.get(row['candidateId'])
        row['editorialCollisionAssessment'] = assessment
        if assessment:
            assert set(assessment['existingKeys']) <= keys
            if assessment['classification'] == 'reject-brand-specific':
                row['proposedImportStatus'] = 'reject-brand-specific'
            elif assessment['classification'] == 'probable-existing-concept':
                row['proposedImportStatus'] = 'defer-probable-duplicate'
    # Surface cross-source candidate collisions too; no automatic identity merges.
    pairs = []
    for index, left in enumerate(candidates):
        for right in candidates[index + 1:]:
            a, b = words(left['proposedTurkishName']), words(right['proposedTurkishName'])
            if normalize(left['proposedTurkishName']) == normalize(right['proposedTurkishName']) or (len(a & b) >= 2 and len(a & b) / max(len(a | b), 1) >= .7):
                pairs.append({'left': left['candidateId'], 'right': right['candidateId'], 'status': 'review-similarity-not-equivalence'})
    prior = json.loads((ROOT / 'catalog/production/m3d1-review.json').read_text())
    historical = json.loads((ROOT / 'catalog/production/localization-review.json').read_text())
    for r in historical:
        if r['key'] in {'usda-sr-171319', 'usda-sr-170874', 'usda-sr-168412'}:
            prior.append(dict(key=r['key'], id=r['id'], tr=r['existingTr'], en=r['existingEn'], categories=r['categories'], reason=r['reason'], status='terminology-review-needed', humanDecision=None, includedInRelease=True, sourceUrl=f"https://fdc.nal.usda.gov/food-details/{r['key'].split('-')[-1]}/nutrients"))
    assert len(prior) == 7 and all(r['humanDecision'] is None for r in prior)
    summary = dict(baselineRelease=7, productionUnchanged=True, productionIdentities=len(catalog['ingredients']), productionAliases=sum(len(i['aliases']) for i in catalog['ingredients']), productionCategories=len(catalog['categories']), candidates=len(candidates),
                   bySource=dict(Counter(r['source'] for r in candidates)), byStatus=dict(Counter(r['proposedImportStatus'] for r in candidates)),
                   byType=dict(Counter(r['ingredientType'] for r in candidates)), candidateCategoryMemberships=dict(Counter(c for r in candidates for c in r['proposedCategories'])),
                   higherConfidenceByType=dict(Counter(r['ingredientType'] for r in candidates if r['proposedImportStatus']=='proposed-after-editorial-review')),
                   candidatesWithCollisionSignals=sum(bool(r['collisions']) for r in candidates), priorPending=7, authorizedImports=0,
                   evidenceSha256=hashlib.sha256((OUT/'source-records.json').read_bytes()).hexdigest(),
                   selectionSha256=hashlib.sha256((OUT/'selection.tsv').read_bytes()).hexdigest())
    for filename, value in [('candidates.json', candidates), ('prior-review.json', prior), ('quality-report.json', summary), ('candidate-pairs.json', pairs)]:
        content = encoded(value)
        if args.check:
            assert (OUT / filename).read_text() == content, f'Stale {filename}'
        else:
            (OUT / filename).write_text(content)
    check_baseline()
    print(json.dumps(summary, ensure_ascii=False))


if __name__ == '__main__':
    main()
