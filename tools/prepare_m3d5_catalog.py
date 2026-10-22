#!/usr/bin/env python3
"""Deterministically build beverage Release 11 from archived Release 10."""
import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path

from prepare_validation_catalog import encoded, stable
from prepare_m3d5_candidates import BASE_MANIFEST_SHA, build as candidate_build, load, normalize

ROOT = Path(__file__).resolve().parents[1]
OLD = ROOT / 'catalog/releases/10'
PREP = ROOT / 'catalog/preparation/m3d5'
OUT = ROOT / 'catalog/production'
SR_SOURCE, WIKI_SOURCE, CURATION_SOURCE = ('usda-sr-legacy-m3d5', 'wikidata-m3d5', 'recipeatlas-curation-m3d5')
SR_VERSION, WIKI_VERSION, CURATION_VERSION = ('2018-04-selection-8', 'entity-revisions-2026-10-22-1', 'production-8')


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def build():
    assert digest((OLD / 'manifest.json').read_bytes()) == BASE_MANIFEST_SHA
    old_manifest, old = load(OLD / 'manifest.json'), load(OLD / 'ingredients.json')
    assert old_manifest['version'] == 10 and len(old['ingredients']) == 795
    for artifact in [old_manifest['artifact'], *old_manifest['evidence'].values()]:
        assert digest((OLD / artifact['file']).read_bytes()) == artifact['sha256'], 'Archived Release 10 evidence changed'
    prep = load(PREP / 'manifest.json')
    assert prep['baselineManifestSha256'] == BASE_MANIFEST_SHA
    for name, sha in prep['outputSha256'].items():
        assert digest((PREP / name).read_bytes()) == sha, f'Candidate artifact changed: {name}'
    assert digest((PREP / 'selection.tsv').read_bytes()) == prep['selectionSha256']
    assert digest((PREP / 'wikidata-index.json').read_bytes()) == prep['wikidataIndexSha256']
    sr_records = {row['fdcId']: row['description'] for row in load(PREP / 'sr-records.json')}
    for path, raw in candidate_build(sr_records).items():
        assert path.read_bytes() == raw, f'Candidate screening no longer reproducible: {path}'
    candidates = load(PREP / 'candidates.json')
    counts = Counter(c['decision'] for c in candidates)
    assert len(candidates) == 85 and counts == {'ACCEPT': 59, 'DEFER': 16, 'EXISTING': 10}
    baseline_keys = {item['key'] for item in old['ingredients']}
    baseline_ids = {item['id'] for item in old['ingredients']}
    labels = {(locale, normalize(locale, name)) for item in old['ingredients']
              for locale, name in item['names'].items()}
    labels |= {(a['locale'], normalize(a['locale'], a['name'])) for item in old['ingredients'] for a in item['aliases']}
    categories = {item['key'] for item in old['categories']}
    additions, evidence = [], {SR_SOURCE: [], WIKI_SOURCE: [], CURATION_SOURCE: []}
    for candidate in candidates:
        if candidate['decision'] != 'ACCEPT':
            continue
        source = SR_SOURCE if candidate['source'] == 'sr' else WIKI_SOURCE
        version = SR_VERSION if source == SR_SOURCE else WIKI_VERSION
        source_id = candidate['sourceIdentifier']
        key = ('usda-sr-' if source == SR_SOURCE else 'wikidata-') + source_id.lower()
        assert key not in baseline_keys and stable('ingredient', key) not in baseline_ids
        assert candidate['sourceEvidence']['externalId'] == source_id
        assert candidate['sourceEvidence']['description'] == candidate['sourceDescription']
        assert candidate['categories'] and set(candidate['categories']) <= categories
        assert candidate['observations'] == [] and not candidate['humanLinguisticApproval']
        for locale, name in (('tr', candidate['proposedTurkish']), ('en', candidate['proposedEnglish'])):
            owner = (locale, normalize(locale, name))
            assert owner not in labels, f'Catalog label collision: {name}'
            labels.add(owner)
        unit = candidate['preferredUnit']
        assert unit in ('g', 'mL')
        authored = dict(tr=candidate['proposedTurkish'], en=candidate['proposedEnglish'],
                        aliasesTr=[], aliasesEn=[], categories=candidate['categories'], preferredUnit=unit,
                        dimensions=['volume' if unit == 'mL' else 'mass'],
                        basis='Autonomous editorial label against pinned source identity; not an official translation or independent human linguistic approval')
        authored_description = json.dumps(authored, ensure_ascii=False, sort_keys=True, separators=(',', ':'))
        evidence[source].append(candidate['sourceEvidence'])
        evidence[CURATION_SOURCE].append(dict(externalId=key, description=authored_description))
        additions.append(dict(id=stable('ingredient', key), key=key, canonicalName=candidate['proposedEnglish'],
                              ingredientType='beverage', names=dict(tr=candidate['proposedTurkish'],
                                                                   en=candidate['proposedEnglish']),
                              aliases=[], categories=candidate['categories'], preferredUnit=unit,
                              dimensions=authored['dimensions'], relations=[],
                              provenance=[dict(sourceId=source, sourceVersion=version, externalId=source_id,
                                               description=candidate['sourceDescription']),
                                          dict(sourceId=CURATION_SOURCE, sourceVersion=CURATION_VERSION,
                                               externalId=key, description=authored_description)],
                              observations=[]))
    assert len(additions) == 59 and len(evidence[SR_SOURCE]) == 27 and len(evidence[WIKI_SOURCE]) == 32
    data = dict(categories=old['categories'], ingredients=sorted(old['ingredients'] + additions, key=lambda i: i['key']))
    outputs, descriptors = {}, {}
    for source, filename in ((SR_SOURCE, 'm3d5-sr-records.json'),
                             (WIKI_SOURCE, 'm3d5-wikidata-records.json'),
                             (CURATION_SOURCE, 'm3d5-curation-records.json')):
        evidence[source].sort(key=lambda item: item['externalId'])
        raw = encoded(evidence[source])
        outputs[OUT / filename] = raw
        descriptors[source] = dict(file=filename, sha256=digest(raw))
    ingredient_raw = encoded(data)
    outputs[OUT / 'ingredients.json'] = ingredient_raw
    new_sources = [
        dict(id=SR_SOURCE, version=SR_VERSION, name='USDA SR Legacy April 2018 — M3D-5 beverage selection',
             url='https://fdc.nal.usda.gov/download-datasets/', license='CC0-1.0',
             attribution='USDA Agricultural Research Service, FoodData Central, SR Legacy April 2018; selected descriptors unchanged. Project selection version is distinct from upstream release.',
             retrievedAt='2026-10-22', artifactSha256=descriptors[SR_SOURCE]['sha256']),
        dict(id=WIKI_SOURCE, version=WIKI_VERSION, name='Wikidata selected beverage entity revisions — M3D-5',
             url='https://www.wikidata.org/wiki/Wikidata:Database_download', license='CC0-1.0',
             attribution='Wikidata contributors; selected structured entity labels and descriptions at pinned QID revisions. Project culinary classification and translations are separate.',
             retrievedAt='2026-10-22', artifactSha256=descriptors[WIKI_SOURCE]['sha256']),
        dict(id=CURATION_SOURCE, version=CURATION_VERSION, name='Tarif Atlası M3D-5 beverage editorial labels',
             url='https://github.com/fatihemres/personal-recipe-library', license='CC0-1.0',
             attribution='Project-authored bilingual culinary labels and classification; no empirical metadata or independent human linguistic approval.',
             retrievedAt='2026-10-22', artifactSha256=descriptors[CURATION_SOURCE]['sha256']),
    ]
    manifest = dict(formatVersion=1, dataset='recipeatlas-ingredients', version=11, purpose='production',
                    artifact=dict(file='ingredients.json', sha256=digest(ingredient_raw)),
                    evidence=old_manifest['evidence'] | descriptors,
                    sources=old_manifest['sources'] + new_sources)
    outputs[OUT / 'manifest.json'] = encoded(manifest)
    report = dict(dataset='recipeatlas-ingredients', version=11, release=11, previousRelease=10,
                  previousIngredientCount=795, selectedRecords=len(data['ingredients']),
                  productionReadyRecords=len(data['ingredients']), candidatesEvaluated=len(candidates),
                  added=len(additions), total=len(data['ingredients']),
                  aliases=sum(len(i['aliases']) for i in data['ingredients']),
                  aliasCount=sum(len(i['aliases']) for i in data['ingredients']),
                  categoryNodes=len(data['categories']),
                  ingredientTypeCounts=dict(Counter(i['ingredientType'] for i in data['ingredients'])),
                  categoryMembershipCounts=dict(sorted(Counter(c for i in data['ingredients'] for c in i['categories']).items())),
                  newCategoryMemberships=dict(sorted(Counter(c for i in additions for c in i['categories']).items())),
                  decisionCounts=dict(counts), acceptedKeys=[i['key'] for i in additions],
                  sourceProvenanceCounts=dict(Counter(p['sourceId'] for i in data['ingredients'] for p in i['provenance'])),
                  independentlyHumanApprovedAdditions=0, inheritedRelease10ManifestSha256=BASE_MANIFEST_SHA)
    outputs[OUT / 'coverage.json'] = encoded(report)
    outputs[OUT / 'm3d5-decisions.json'] = encoded(candidates)
    outputs[OUT / 'm3d5-quality-report.json'] = encoded(report | {
        'auditMeaning': 'Decision/coverage summary; run tools/audit_catalog.py for structural and alias findings.'})
    return outputs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    outputs = build()
    for path, raw in outputs.items():
        if args.check:
            assert path.read_bytes() == raw, f'Stale Release 11 artifact: {path}'
        else:
            path.write_bytes(raw)
    coverage = json.loads(outputs[OUT / 'coverage.json'])
    print(f"Release 11: {coverage['added']} additions, {coverage['total']} identities, "
          f"{coverage['ingredientTypeCounts']['beverage']} beverage, {coverage['aliases']} aliases")


if __name__ == '__main__':
    main()
