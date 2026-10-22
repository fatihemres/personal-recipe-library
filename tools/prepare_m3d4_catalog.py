#!/usr/bin/env python3
"""Deterministically build Release 10 from archived Release 9 and screened evidence."""
import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path

from prepare_validation_catalog import encoded, stable
from prepare_m3d4_candidates import BASE_MANIFEST_SHA, normalize

ROOT = Path(__file__).resolve().parents[1]
OLD = ROOT / 'catalog/releases/9'
PREP = ROOT / 'catalog/preparation/m3d4'
OUT = ROOT / 'catalog/releases/10'
FOUNDATION_SOURCE = 'usda-foundation-m3d4'
WIKIDATA_SOURCE = 'wikidata-m3d4'
CURATION_SOURCE = 'recipeatlas-curation-m3d4'
FOUNDATION_VERSION = '2026-04-30-selection-1'
WIKIDATA_VERSION = 'entity-revisions-2026-10-19-1'
CURATION_VERSION = 'production-7'


def load(path):
    return json.loads(path.read_text(encoding='utf-8'))


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def build():
    assert digest((OLD / 'manifest.json').read_bytes()) == BASE_MANIFEST_SHA, 'Archived Release 9 manifest changed'
    old_manifest, old = load(OLD / 'manifest.json'), load(OLD / 'ingredients.json')
    assert old_manifest['version'] == 9 and len(old['ingredients']) == 745
    for artifact in [old_manifest['artifact'], *old_manifest['evidence'].values()]:
        assert digest((OLD / artifact['file']).read_bytes()) == artifact['sha256'], 'Archived Release 9 evidence changed'
    prep = load(PREP / 'manifest.json')
    assert prep['baselineManifestSha256'] == BASE_MANIFEST_SHA
    for name, sha in prep['outputSha256'].items():
        assert digest((PREP / name).read_bytes()) == sha, f'Candidate artifact changed: {name}'
    assert digest((PREP / 'selection.tsv').read_bytes()) == prep['selectionSha256']
    assert digest((PREP / 'wikidata-index.json').read_bytes()) == prep['wikidataIndexSha256']
    candidates = load(PREP / 'candidates.json')
    assert len(candidates) == 65 and Counter(c['decision'] for c in candidates) == {'ACCEPT': 50, 'DEFER': 9, 'EXISTING': 4, 'REJECT': 2}
    baseline_keys = {i['key'] for i in old['ingredients']}
    baseline_ids = {i['id'] for i in old['ingredients']}
    labels = {(locale, normalize(locale, name)) for i in old['ingredients'] for locale, name in i['names'].items()}
    labels |= {(a['locale'], normalize(a['locale'], a['name'])) for i in old['ingredients'] for a in i['aliases']}
    categories = {c['key'] for c in old['categories']}
    additions, evidence = [], {FOUNDATION_SOURCE: [], WIKIDATA_SOURCE: [], CURATION_SOURCE: []}
    for candidate in candidates:
        if candidate['decision'] != 'ACCEPT':
            continue
        source = FOUNDATION_SOURCE if candidate['source'] == 'foundation' else WIKIDATA_SOURCE
        version = FOUNDATION_VERSION if source == FOUNDATION_SOURCE else WIKIDATA_VERSION
        source_id = candidate['sourceIdentifier']
        key = ('usda-foundation-' if source == FOUNDATION_SOURCE else 'wikidata-') + source_id.lower()
        assert key not in baseline_keys and stable('ingredient', key) not in baseline_ids
        assert candidate['sourceEvidence']['externalId'] == source_id
        assert candidate['sourceEvidence']['description'] == candidate['sourceDescription']
        assert candidate['categories'] and set(candidate['categories']) <= categories
        assert candidate['observations'] == [] and not candidate['humanLinguisticApproval']
        for locale, name in [('tr', candidate['proposedTurkish']), ('en', candidate['proposedEnglish'])]:
            owner = (locale, normalize(locale, name))
            assert owner not in labels, f'Catalog label collision: {name}'
            labels.add(owner)
        unit = candidate['preferredUnit']
        assert unit in ('g', 'mL')
        authored = dict(tr=candidate['proposedTurkish'], en=candidate['proposedEnglish'], aliasesTr=[], aliasesEn=[],
                        categories=candidate['categories'], preferredUnit=unit,
                        dimensions=['volume' if unit == 'mL' else 'mass'],
                        basis='Autonomous editorial label against pinned source identity; not an official translation or independent human linguistic approval')
        authored_description = json.dumps(authored, ensure_ascii=False, sort_keys=True, separators=(',', ':'))
        evidence[source].append(candidate['sourceEvidence'])
        evidence[CURATION_SOURCE].append(dict(externalId=key, description=authored_description))
        ingredient_type = 'beverage' if unit == 'mL' and any(c in candidate['categories'] for c in ('juices', 'other-beverages')) else 'food'
        additions.append(dict(id=stable('ingredient', key), key=key, canonicalName=candidate['proposedEnglish'],
                              ingredientType=ingredient_type,
                              names=dict(tr=candidate['proposedTurkish'], en=candidate['proposedEnglish']), aliases=[],
                              categories=candidate['categories'], preferredUnit=unit,
                              dimensions=authored['dimensions'], relations=[],
                              provenance=[dict(sourceId=source, sourceVersion=version, externalId=source_id,
                                               description=candidate['sourceDescription']),
                                          dict(sourceId=CURATION_SOURCE, sourceVersion=CURATION_VERSION,
                                               externalId=key, description=authored_description)],
                              observations=[]))
    assert len(additions) == 50 and len(evidence[FOUNDATION_SOURCE]) == 35 and len(evidence[WIKIDATA_SOURCE]) == 15
    data = dict(categories=old['categories'], ingredients=sorted(old['ingredients'] + additions, key=lambda i: i['key']))
    outputs, descriptors = {}, {}
    for source, filename in [(FOUNDATION_SOURCE, 'm3d4-foundation-records.json'),
                             (WIKIDATA_SOURCE, 'm3d4-wikidata-records.json'),
                             (CURATION_SOURCE, 'm3d4-curation-records.json')]:
        evidence[source].sort(key=lambda item: item['externalId'])
        raw = encoded(evidence[source])
        outputs[OUT / filename] = raw
        descriptors[source] = dict(file=filename, sha256=digest(raw))
    ingredient_raw = encoded(data)
    outputs[OUT / 'ingredients.json'] = ingredient_raw
    new_sources = [
        dict(id=FOUNDATION_SOURCE, version=FOUNDATION_VERSION, name='USDA Foundation Foods April 2026 — M3D-4 selection',
             url='https://fdc.nal.usda.gov/download-datasets/', license='CC0-1.0',
             attribution='USDA Agricultural Research Service, FoodData Central, Foundation Foods April 2026; selected descriptors unchanged. Project selection version is distinct from upstream release.',
             retrievedAt='2026-10-19', artifactSha256=descriptors[FOUNDATION_SOURCE]['sha256']),
        dict(id=WIKIDATA_SOURCE, version=WIKIDATA_VERSION, name='Wikidata selected ingredient entity revisions — M3D-4',
             url='https://www.wikidata.org/wiki/Wikidata:Database_download', license='CC0-1.0',
             attribution='Wikidata contributors; selected structured entity labels and descriptions at pinned QID revisions. Project culinary classification and translations are separate.',
             retrievedAt='2026-10-19', artifactSha256=descriptors[WIKIDATA_SOURCE]['sha256']),
        dict(id=CURATION_SOURCE, version=CURATION_VERSION, name='Tarif Atlası M3D-4 editorial labels',
             url='https://github.com/fatihemres/personal-recipe-library', license='CC0-1.0',
             attribution='Project-authored bilingual culinary labels and classification; no empirical metadata or independent human linguistic approval.',
             retrievedAt='2026-10-19', artifactSha256=descriptors[CURATION_SOURCE]['sha256']),
    ]
    manifest = dict(formatVersion=1, dataset='recipeatlas-ingredients', version=10, purpose='production',
                    artifact=dict(file='ingredients.json', sha256=digest(ingredient_raw)),
                    evidence=old_manifest['evidence'] | descriptors,
                    sources=old_manifest['sources'] + new_sources)
    outputs[OUT / 'manifest.json'] = encoded(manifest)
    report = dict(dataset='recipeatlas-ingredients', version=10, release=10, previousRelease=9,
                  previousIngredientCount=745, selectedRecords=795, productionReadyRecords=795,
                  candidatesEvaluated=len(candidates), added=50, total=len(data['ingredients']),
                  aliases=sum(len(i['aliases']) for i in data['ingredients']), aliasCount=sum(len(i['aliases']) for i in data['ingredients']),
                  categoryNodes=len(data['categories']), ingredientTypeCounts=dict(Counter(i['ingredientType'] for i in data['ingredients'])),
                  categoryMembershipCounts=dict(sorted(Counter(c for i in data['ingredients'] for c in i['categories']).items())),
                  newCategoryMemberships=dict(sorted(Counter(c for i in additions for c in i['categories']).items())),
                  decisionCounts=dict(Counter(c['decision'] for c in candidates)),
                  acceptedKeys=[i['key'] for i in additions],
                  sourceProvenanceCounts=dict(Counter(p['sourceId'] for i in data['ingredients'] for p in i['provenance'])),
                  unresolvedHistoricalReview=7, independentlyHumanApprovedAdditions=0,
                  inheritedRelease9ManifestSha256=BASE_MANIFEST_SHA)
    outputs[OUT / 'coverage.json'] = encoded(report)
    outputs[OUT / 'm3d4-decisions.json'] = encoded(candidates)
    outputs[OUT / 'm3d4-quality-report.json'] = encoded(report | {'auditMeaning': 'Decision/coverage summary; run tools/audit_catalog.py for structural and alias findings.'})
    return outputs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    outputs = build()
    for path, raw in outputs.items():
        if args.check:
            assert path.read_bytes() == raw, f'Stale Release 10 artifact: {path}'
        else:
            path.write_bytes(raw)
    print('Release 10: 50 additions, 795 identities, 269 aliases, 50 categories')


if __name__ == '__main__':
    main()
