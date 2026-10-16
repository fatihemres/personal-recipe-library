#!/usr/bin/env python3
"""Create release 8 from immutable release 7 and the M3D-2A USDA ledger.

Only explicit, auditable editorial decisions can become seed records. This tool
does not download data, edit user profiles, or grant human linguistic approval.
"""
import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path

from prepare_validation_catalog import encoded, stable
from prepare_m3d2a_candidates import normalize

ROOT = Path(__file__).resolve().parents[1]
OLD = ROOT / 'catalog/releases/7'
OUT = ROOT / 'catalog/production'
PREP = ROOT / 'catalog/preparation/m3d2a'
MANIFEST_SHA = '16b4b2a4120aefb28034dfd4142478ef515d4a8473f1e1d791db48aa9535634d'
USDA_SOURCE = 'usda-sr-legacy-m3d2b'
CURATION_SOURCE = 'recipeatlas-curation-m3d2b'
USDA_VERSION = '2018-04-selection-6'
CURATION_VERSION = 'production-5'
# An earlier, locally installed 61-record M3D-1 preview used these source
# versions with different evidence hashes. Source snapshots are immutable, so
# release 8 must use distinct versions for the reviewed 60-record extraction.
REVIEWED_M3D1_VERSIONS = {
    'usda-sr-legacy-m3d1': ('2018-04-selection-5', '2018-04-selection-5-reviewed'),
    'recipeatlas-curation-m3d1': ('production-4', 'production-4-reviewed'),
}

# Individual re-review of the forty M3D-2A questions. These thirteen have
# source-explicit distinctions and cautious Turkish labels. The other 27 retain
# their original concern and are deferred; none is silently merged or renamed.
REVIEW_ACCEPT = {
    '168385': 'Amaranth leaves are botanically distinct from the grape-leaf fuzzy match.',
    '168583': 'Source explicitly identifies raw wasabi root; do not alias imitation wasabi.',
    '169110': 'Yellow juice differs from the existing purple passion-fruit juice.',
    '169129': 'Pitanga preserves the source fruit identity without an uncertain translated common name.',
    '169398': 'Epazote preserves its culinary name without claiming another herb is equivalent.',
    '169985': 'Cassava and the fuzzy-matched mango are unrelated ingredients.',
    '170090': 'Dried agar seaweed is a different form from purified agar powder.',
    '170167': 'Dried peeled chestnuts differ explicitly from raw chestnuts.',
    '170186': 'Black walnut is a source-explicit variety; not an alias of common walnut.',
    '171575': 'Prepared peanut sauce is distinct from its component peanuts.',
    '171595': 'Bottled tomato chili sauce is distinct from unresolved chili seasoning powder.',
    '171871': 'Root beer is explicitly a carbonated soft drink, not beer.',
    '175132': 'Mixed-species raw fish roe is kept distinct from prepared caviar.',
}


def load(path):
    return json.loads(path.read_text(encoding='utf-8'))


def artifact(value):
    raw = encoded(value)
    return raw, {'sha256': hashlib.sha256(raw).hexdigest()}


def build():
    assert hashlib.sha256((OLD / 'manifest.json').read_bytes()).hexdigest() == MANIFEST_SHA, 'Release 7 manifest changed'
    previous_manifest = load(OLD / 'manifest.json')
    previous = load(OLD / 'ingredients.json')
    assert previous_manifest['version'] == 7 and len(previous['ingredients']) == 568
    for descriptor in [previous_manifest['artifact'], *previous_manifest['evidence'].values()]:
        assert hashlib.sha256((OLD / descriptor['file']).read_bytes()).hexdigest() == descriptor['sha256'], 'Release 7 artifact changed'
    for source in previous_manifest['sources']:
        if source['id'] in REVIEWED_M3D1_VERSIONS:
            old_version, reviewed_version = REVIEWED_M3D1_VERSIONS[source['id']]
            assert source['version'] == old_version
            source['version'] = reviewed_version
    for item in previous['ingredients']:
        for provenance in item['provenance']:
            if provenance['sourceId'] in REVIEWED_M3D1_VERSIONS:
                old_version, reviewed_version = REVIEWED_M3D1_VERSIONS[provenance['sourceId']]
                assert provenance['sourceVersion'] == old_version
                provenance['sourceVersion'] = reviewed_version
    catalog = {item['key']: item for item in previous['ingredients']}
    category_keys = {c['key'] for c in previous['categories']}
    preparation_report = load(PREP / 'quality-report.json')
    assert hashlib.sha256((PREP / 'source-records.json').read_bytes()).hexdigest() == preparation_report['evidenceSha256'], 'Pinned candidate evidence changed'
    assert hashlib.sha256((PREP / 'selection.tsv').read_bytes()).hexdigest() == preparation_report['selectionSha256'], 'Candidate selection changed'
    candidates = load(PREP / 'candidates.json')
    source = {(row['source'], row['id']): row for row in load(PREP / 'source-records.json')}
    assert len(candidates) == 194 and sum(c['source'] == 'usda' for c in candidates) == 106
    assert len({c['candidateId'] for c in candidates}) == 194
    assert set(REVIEW_ACCEPT) <= {c['sourceIdentifier'] for c in candidates if c['source'] == 'usda'}
    owners = {}
    for item in previous['ingredients']:
        for locale, name in list(item['names'].items()) + [(a['locale'], a['name']) for a in item['aliases']]:
            owners.setdefault((locale, normalize(name)), set()).add(item['key'])
    previous_usda = {p['externalId'] for item in previous['ingredients'] for p in item['provenance'] if p['sourceId'].startswith('usda-sr-legacy')}
    decisions, additions, usda_evidence, curation_evidence = [], [], [], []
    for candidate in sorted(candidates, key=lambda c: c['candidateId']):
        evidence = source[candidate['source'], candidate['sourceIdentifier']]
        assert candidate['sourceEnglishLabel'] == evidence['label'] and candidate['sourceUrl'] == evidence['url']
        assert not candidate['importAuthorized'] and candidate['humanDecision'] is None and candidate['observations'] == []
        if candidate['source'] == 'foodon':
            status = 'REJECT' if candidate['proposedImportStatus'] == 'reject-brand-specific' else 'EXISTING' if candidate['proposedImportStatus'] == 'defer-probable-duplicate' else 'DEFER'
            reason = 'Brand-specific commercial identity; not a generic culinary ingredient.' if status == 'REJECT' else 'Probable existing concept; cross-source identity review required.' if status == 'EXISTING' else 'Term-level lineage, license and attribution clearance remain incomplete.'
        elif candidate['sourceIdentifier'] in previous_usda:
            status, reason = 'EXISTING', 'USDA source identifier is already mapped to an installed canonical identity.'
        elif candidate['proposedImportStatus'] == 'proposed-after-editorial-review':
            status, reason = 'ACCEPT', 'Exact pinned USDA descriptor, distinct bilingual form, valid categories and no recorded collision or terminology concern; autonomous editorial selection, not human linguistic approval.'
        elif candidate['sourceIdentifier'] in REVIEW_ACCEPT:
            status, reason = 'ACCEPT', REVIEW_ACCEPT[candidate['sourceIdentifier']]
        else:
            status, reason = 'DEFER', candidate['concern'] or 'Possible existing synonym/form or uncertain culinary terminology requires additional identity review.'
        decisions.append(dict(candidateId=candidate['candidateId'], source=candidate['source'], sourceIdentifier=candidate['sourceIdentifier'], sourceUrl=candidate['sourceUrl'], decision=status, reason=reason, reviewedBy='autonomous editorial process', independentlyHumanApproved=False, originalScreening=candidate['proposedImportStatus'], collisionSignals=candidate['collisions'], editorialCollisionAssessment=candidate['editorialCollisionAssessment']))
        if status != 'ACCEPT':
            continue
        assert candidate['source'] == 'usda' and candidate['sourceIdentifier'] not in previous_usda
        key = 'usda-sr-' + candidate['sourceIdentifier']
        categories = candidate['proposedCategories'] + (['other-beverages'] if candidate['sourceIdentifier'] == '174866' else [])
        assert key not in catalog and len(categories) == len(set(categories)) and set(categories) <= category_keys
        assert candidate['sourceEnglishLabel'] == candidate['proposedEnglishName']
        for locale, name in (('tr', candidate['proposedTurkishName']), ('en', candidate['proposedEnglishName'])):
            assert name == name.strip() and name
            owner = (locale, normalize(name))
            assert owner not in owners, f'Catalog name collision: {name} / {owners.get(owner)}'
            owners[owner] = {key}
        # A beverage-making powder or instant preparation is measured by mass.
        dry_preparation = any(term in evidence['label'].lower() for term in ('powder', 'instant'))
        unit = 'mL' if candidate['ingredientType'] == 'beverage' and not dry_preparation else 'g'
        authored = dict(tr=candidate['proposedTurkishName'], en=candidate['proposedEnglishName'], aliasesTr=[], aliasesEn=[], categories=categories, preferredUnit=unit, dimensions=['volume' if unit == 'mL' else 'mass'], basis='Autonomous project editorial selection against pinned USDA descriptor; not a USDA translation or independent human linguistic approval')
        authored_description = json.dumps(authored, ensure_ascii=False, sort_keys=True, separators=(',', ':'))
        usda_evidence.append(dict(externalId=candidate['sourceIdentifier'], description=evidence['label']))
        curation_evidence.append(dict(externalId=key, description=authored_description))
        additions.append(dict(id=stable('ingredient', key), key=key, canonicalName=candidate['proposedEnglishName'], ingredientType=candidate['ingredientType'], names=dict(tr=candidate['proposedTurkishName'], en=candidate['proposedEnglishName']), aliases=[], categories=categories, preferredUnit=unit, dimensions=authored['dimensions'], relations=[], provenance=[dict(sourceId=USDA_SOURCE, sourceVersion=USDA_VERSION, externalId=candidate['sourceIdentifier'], description=evidence['label']), dict(sourceId=CURATION_SOURCE, sourceVersion=CURATION_VERSION, externalId=key, description=authored_description)], observations=[]))
    assert len(decisions) == 194 and Counter(d['decision'] for d in decisions if d['source'] == 'usda') == {'ACCEPT': 79, 'DEFER': 27}
    assert Counter(d['decision'] for d in decisions if d['source'] == 'foodon') == {'DEFER': 84, 'EXISTING': 3, 'REJECT': 1}
    assert len(additions) == 79 and len({i['id'] for i in additions}) == 79
    ingredients = dict(categories=previous['categories'], ingredients=sorted(previous['ingredients'] + additions, key=lambda i: i['key']))
    artifacts = {}
    outputs = {}
    for filename, value in [('m3d2b-usda-records.json', usda_evidence), ('m3d2b-curation-records.json', curation_evidence), ('ingredients.json', ingredients)]:
        raw, meta = artifact(value)
        outputs[OUT / filename] = raw
        artifacts[filename] = dict(file=filename, **meta)
    new_sources = [dict(id=USDA_SOURCE, version=USDA_VERSION, name='USDA SR Legacy April 2018 — M3D-2B selection', url='https://fdc.nal.usda.gov/download-datasets/', license='CC0-1.0', attribution='USDA Agricultural Research Service, FoodData Central, SR Legacy April 2018; selected descriptors unchanged. Selection version is project-owned.', retrievedAt='2026-10-15', artifactSha256=artifacts['m3d2b-usda-records.json']['sha256']), dict(id=CURATION_SOURCE, version=CURATION_VERSION, name='Tarif Atlası M3D-2B editorial labels', url='https://github.com/fatihemres/personal-recipe-library', license='CC0-1.0', attribution='Project-authored bilingual labels and classifications; no empirical metadata or independent linguistic approval.', retrievedAt='2026-10-15', artifactSha256=artifacts['m3d2b-curation-records.json']['sha256'])]
    manifest = dict(formatVersion=1, dataset='recipeatlas-ingredients', version=8, purpose='production', artifact=artifacts['ingredients.json'], evidence=previous_manifest['evidence'] | {USDA_SOURCE: artifacts['m3d2b-usda-records.json'], CURATION_SOURCE: artifacts['m3d2b-curation-records.json']}, sources=previous_manifest['sources'] + new_sources)
    outputs[OUT / 'manifest.json'] = encoded(manifest)
    output_counts = Counter(i['ingredientType'] for i in ingredients['ingredients'])
    report = dict(dataset='recipeatlas-ingredients', version=8, release=8, previousRelease=7, previousIngredientCount=len(previous['ingredients']), selectedRecords=len(ingredients['ingredients']), productionReadyRecords=len(ingredients['ingredients']), candidatesEvaluated=len(decisions), decisionCounts=dict(Counter(d['decision'] for d in decisions)), bySource={s: dict(Counter(d['decision'] for d in decisions if d['source'] == s)) for s in ('usda','foodon')}, added=len(additions), total=len(ingredients['ingredients']), ingredientTypeCounts=dict(output_counts), aliases=sum(len(i['aliases']) for i in ingredients['ingredients']), aliasCount=sum(len(i['aliases']) for i in ingredients['ingredients']), categoryNodes=len(ingredients['categories']), categoryMembershipCounts=dict(sorted(Counter(c for i in ingredients['ingredients'] for c in i['categories']).items())), newCategoryMemberships=dict(sorted(Counter(c for i in additions for c in i['categories']).items())), sourceProvenanceCounts=dict(Counter(p['sourceId'] for i in ingredients['ingredients'] for p in i['provenance'])), unresolvedHistoricalReview=7, independentlyHumanApprovedAdditions=0, missingFactualMetadata={field:len(ingredients['ingredients']) for field in ('nutrition','density','allergens','abv')}, inheritedRelease7ManifestSha256=MANIFEST_SHA)
    outputs[OUT / 'coverage.json'] = encoded(report)
    outputs[OUT / 'm3d2b-decisions.json'] = encoded(decisions)
    outputs[OUT / 'm3d2b-quality-report.json'] = encoded(report | {'acceptedKeys': [i['key'] for i in additions], 'auditMeaning': 'Package and decision counts only; run tools/audit_catalog.py for structural and collision findings.'})
    return outputs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    outputs = build()
    if args.check:
        for path, expected in outputs.items():
            assert path.read_bytes() == expected, f'Stale release 8 artifact: {path}'
    else:
        for path, content in outputs.items():
            path.write_bytes(content)
    print('Release 8: 79 USDA additions, 647 identities, 269 aliases, 50 categories; FoodOn excluded')


if __name__ == '__main__':
    main()
