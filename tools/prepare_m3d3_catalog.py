#!/usr/bin/env python3
"""Build release 9 solely from archived release 8 and pinned M3D-3 decisions."""
import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path

from prepare_validation_catalog import encoded, stable
from prepare_m3d3_candidates import normalize

ROOT = Path(__file__).resolve().parents[1]
OLD = ROOT / 'catalog/releases/8'
PREP = ROOT / 'catalog/preparation/m3d3'
OUT = ROOT / 'catalog/production'
OLD_SHA = '9bad5c51370bb6d13c2bbd313b3279244a58b8d57e1a720be1a9d4e75f213e65'
USDA_SOURCE = 'usda-sr-legacy-m3d3'
CURATION_SOURCE = 'recipeatlas-curation-m3d3'
USDA_VERSION = '2018-04-selection-7'
CURATION_VERSION = 'production-6'


def load(path):
    return json.loads(path.read_text(encoding='utf-8'))


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def build():
    assert digest((OLD / 'manifest.json').read_bytes()) == OLD_SHA, 'Release 8 manifest changed'
    old_manifest = load(OLD / 'manifest.json')
    old = load(OLD / 'ingredients.json')
    assert old_manifest['version'] == 8 and len(old['ingredients']) == 647
    for descriptor in [old_manifest['artifact'], *old_manifest['evidence'].values()]:
        assert digest((OLD / descriptor['file']).read_bytes()) == descriptor['sha256'], 'Archived artifact changed'
    prep_manifest = load(PREP / 'manifest.json')
    assert prep_manifest['baselineManifestSha256'] == OLD_SHA
    for name, key in [('selection.tsv', 'selectionSha256'), ('other-decisions.tsv', 'otherDecisionsSha256'), ('candidates.json', 'candidatesSha256'), ('source-records.json', 'sourceRecordsSha256')]:
        assert digest((PREP / name).read_bytes()) == prep_manifest[key], f'Candidate input changed: {name}'
    candidates = load(PREP / 'candidates.json')
    facts = {r['fdc_id']: r for r in load(PREP / 'source-records.json')}
    assert len(candidates) == 131 and Counter(c['decision'] for c in candidates) == {'ACCEPT': 98, 'DEFER': 29, 'EXISTING': 3, 'REJECT': 1}
    assert len(facts) == len(candidates)
    existing_keys = {i['key'] for i in old['ingredients']}
    existing_ids = {i['id'] for i in old['ingredients']}
    existing_source_ids = {p['externalId'] for i in old['ingredients'] for p in i['provenance'] if p['sourceId'].startswith('usda-sr-legacy')}
    owners = {(locale, normalize(locale, name)) for i in old['ingredients'] for locale, name in i['names'].items()}
    owners |= {(a['locale'], normalize(a['locale'], a['name'])) for i in old['ingredients'] for a in i['aliases']}
    categories = {c['key'] for c in old['categories']}
    additions, usda_evidence, curation_evidence = [], [], []
    for candidate in candidates:
        if candidate['decision'] != 'ACCEPT':
            continue
        external_id = candidate['sourceIdentifier']
        fact = facts[external_id]
        key = 'usda-sr-' + external_id
        assert fact['description'] == candidate['sourceDescription'] and external_id not in existing_source_ids
        assert key not in existing_keys and stable('ingredient', key) not in existing_ids
        assert set(candidate['categories']) <= categories and candidate['categories']
        assert candidate['observations'] == [] and candidate['humanLinguisticApproval'] is False
        for locale, name in [('tr', candidate['proposedTurkish']), ('en', candidate['proposedEnglish'])]:
            owner = (locale, normalize(locale, name))
            assert owner not in owners, f'Catalog label collision: {name}'
            owners.add(owner)
        unit = candidate['preferredUnit']
        assert unit in ('g', 'mL')
        authored = dict(tr=candidate['proposedTurkish'], en=candidate['proposedEnglish'], aliasesTr=[], aliasesEn=[], categories=candidate['categories'], preferredUnit=unit, dimensions=['volume' if unit == 'mL' else 'mass'], basis='Autonomous editorial label against pinned USDA descriptor; not a USDA translation or independent human linguistic approval')
        authored_description = json.dumps(authored, ensure_ascii=False, sort_keys=True, separators=(',', ':'))
        usda_evidence.append(dict(externalId=external_id, description=fact['description']))
        curation_evidence.append(dict(externalId=key, description=authored_description))
        additions.append(dict(id=stable('ingredient', key), key=key, canonicalName=candidate['proposedEnglish'], ingredientType=candidate['ingredientType'], names=dict(tr=candidate['proposedTurkish'], en=candidate['proposedEnglish']), aliases=[], categories=candidate['categories'], preferredUnit=unit, dimensions=authored['dimensions'], relations=[], provenance=[dict(sourceId=USDA_SOURCE, sourceVersion=USDA_VERSION, externalId=external_id, description=fact['description']), dict(sourceId=CURATION_SOURCE, sourceVersion=CURATION_VERSION, externalId=key, description=authored_description)], observations=[]))
    assert len(additions) == 98
    data = dict(categories=old['categories'], ingredients=sorted(old['ingredients'] + additions, key=lambda i: i['key']))
    outputs, descriptors = {}, {}
    for filename, value in [('m3d3-usda-records.json', usda_evidence), ('m3d3-curation-records.json', curation_evidence), ('ingredients.json', data)]:
        raw = encoded(value)
        outputs[OUT / filename] = raw
        descriptors[filename] = dict(file=filename, sha256=digest(raw))
    new_sources = [
        dict(id=USDA_SOURCE, version=USDA_VERSION, name='USDA SR Legacy April 2018 — M3D-3 selection', url='https://fdc.nal.usda.gov/download-datasets/', license='CC0-1.0', attribution='USDA Agricultural Research Service, FoodData Central, SR Legacy April 2018; selected descriptors unchanged. Selection version is project-owned.', retrievedAt='2026-10-16', artifactSha256=descriptors['m3d3-usda-records.json']['sha256']),
        dict(id=CURATION_SOURCE, version=CURATION_VERSION, name='Tarif Atlası M3D-3 editorial labels', url='https://github.com/fatihemres/personal-recipe-library', license='CC0-1.0', attribution='Project-authored bilingual labels and classifications; no empirical metadata or independent linguistic approval.', retrievedAt='2026-10-16', artifactSha256=descriptors['m3d3-curation-records.json']['sha256']),
    ]
    manifest = dict(formatVersion=1, dataset='recipeatlas-ingredients', version=9, purpose='production', artifact=descriptors['ingredients.json'], evidence=old_manifest['evidence'] | {USDA_SOURCE: descriptors['m3d3-usda-records.json'], CURATION_SOURCE: descriptors['m3d3-curation-records.json']}, sources=old_manifest['sources'] + new_sources)
    outputs[OUT / 'manifest.json'] = encoded(manifest)
    report = dict(dataset='recipeatlas-ingredients', version=9, release=9, previousRelease=8, previousIngredientCount=647, selectedRecords=len(data['ingredients']), productionReadyRecords=len(data['ingredients']), candidatesEvaluated=len(candidates), added=98, total=len(data['ingredients']), aliases=sum(len(i['aliases']) for i in data['ingredients']), aliasCount=sum(len(i['aliases']) for i in data['ingredients']), categoryNodes=len(data['categories']), ingredientTypeCounts=dict(Counter(i['ingredientType'] for i in data['ingredients'])), categoryMembershipCounts=dict(sorted(Counter(c for i in data['ingredients'] for c in i['categories']).items())), newCategoryMemberships=dict(sorted(Counter(c for i in additions for c in i['categories']).items())), decisionCounts=dict(Counter(c['decision'] for c in candidates)), acceptedKeys=[i['key'] for i in additions], sourceProvenanceCounts=dict(Counter(p['sourceId'] for i in data['ingredients'] for p in i['provenance'])), unresolvedHistoricalReview=7, independentlyHumanApprovedAdditions=0, inheritedRelease8ManifestSha256=OLD_SHA)
    assert report['total'] == 745 and report['aliases'] == 269 and report['categoryNodes'] == 50
    outputs[OUT / 'coverage.json'] = encoded(report)
    outputs[OUT / 'm3d3-decisions.json'] = encoded(candidates)
    outputs[OUT / 'm3d3-quality-report.json'] = encoded(report | {'auditMeaning': 'Decision and coverage summary; run tools/audit_catalog.py for structural and alias findings.'})
    return outputs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    outputs = build()
    for path, content in outputs.items():
        if args.check:
            assert path.read_bytes() == content, f'Stale release 9 artifact: {path}'
        else:
            path.write_bytes(content)
    print('Release 9: 98 USDA additions, 745 identities, 269 aliases, 50 categories')


if __name__ == '__main__':
    main()
