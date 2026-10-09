#!/usr/bin/env python3
"""Deterministic audit of all production labels/identities, not linguistic certification.
Run normally to generate reports, or --check to require committed report freshness.
"""
import collections
import hashlib
import re
import json
import pathlib
import sys
import unicodedata

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / 'catalog/production'


def normalize(locale, text):
    text = unicodedata.normalize('NFC', text)
    if locale == 'tr':
        text = text.replace('I', 'ı').replace('İ', 'i')
    return ' '.join(text.lower().split())


def audit():
    data = json.loads((OUT / 'ingredients.json').read_text())
    old = json.loads((ROOT / 'catalog/releases/3/ingredients.json').read_text())
    manifest = json.loads((OUT / 'manifest.json').read_text())
    entries = {i['key']: i for i in data['ingredients']}
    assert len(entries) == len(data['ingredients']), 'Duplicate canonical keys'
    assert len({i['id'] for i in entries.values()}) == len(entries), 'Duplicate stable IDs'
    for i in old['ingredients']:
        assert i['key'] in entries and entries[i['key']]['id'] == i['id'], 'Lost historical identity'
    category_keys = {c['key'] for c in data['categories']}
    categories = {c['key']: c for c in data['categories']}
    assert len(categories) == len(data['categories']), 'Duplicate category keys'
    for category in categories.values():
        chain = set()
        current = category['key']
        while current is not None:
            assert current in categories, 'Broken category parent'
            assert current not in chain, 'Category cycle'
            chain.add(current)
            current = categories[current].get('parent')
    sources = {(s['id'], s['version']): s for s in manifest['sources']}
    registry = {s['id']: s for s in json.loads((ROOT / 'catalog/sources.json').read_text())['sources']}
    references = {r['url']: r for r in json.loads((ROOT / 'catalog/reference-sources.json').read_text())['references']}
    for source in manifest['sources']:
        approved = registry[source['id']]
        assert approved['seedApproved'] and source['license'] == approved['license'], 'Unapproved source or license'
        assert source['version'] in approved.get('pinnedVersions', [approved['pinnedVersion']]), 'Unapproved source version'
    for authored in json.loads((OUT / 'reference-curation.json').read_text()):
        assert authored['references'], 'Missing primary identity reference'
        for url in authored['references']:
            assert authored['key'] in references[url]['identities'], 'Unregistered primary identity reference'
    evidence = {}
    for source in manifest['sources']:
        artifact = manifest['evidence'][source['id']]
        raw = (OUT / artifact['file']).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == artifact['sha256'] == source['artifactSha256'], 'Evidence checksum mismatch'
        records = json.loads(raw)
        assert len({r['externalId'] for r in records}) == len(records), 'Duplicate evidence identities'
        evidence[source['id']] = {r['externalId']: r['description'] for r in records}
    assert hashlib.sha256((OUT / manifest['artifact']['file']).read_bytes()).hexdigest() == manifest['artifact']['sha256'], 'Ingredient checksum mismatch'
    owners = collections.defaultdict(set)
    source_owners = collections.defaultdict(set)
    warnings = []
    concepts = collections.defaultdict(list)
    errors = []
    for key, i in entries.items():
        for provenance in i['provenance']:
            assert (provenance['sourceId'], provenance['sourceVersion']) in sources, 'Unknown provenance source/version'
            assert evidence[provenance['sourceId']].get(provenance['externalId']) == provenance['description'], 'Unverified source identifier or descriptor'
            if provenance['sourceId'] == 'usda-sr-legacy':
                assert re.fullmatch(r'[0-9]{6}', provenance['externalId']), 'Invalid USDA identifier'
            source_owners[(provenance['sourceId'], provenance['externalId'])].add(key)
        assert i['ingredientType'] in ('food', 'beverage', 'alcohol', 'garnish', 'other'), 'Invalid ingredient type'
        assert not set(i['categories']) - category_keys, 'Invalid ingredient category reference'
        for related in i['relations']:
            assert related['key'] in entries and related['key'] != key, 'Broken ingredient relationship'
        main_categories = [c for c in i['categories'] if c not in ('garnishes','beverage','international-specialty','turkish-regional')]
        if i['ingredientType'] == 'food' and any(c in ('spirits','wines','beers','liqueurs','bitters') for c in main_categories):
            errors.append({'key':key,'problem':'Food type for alcoholic beverage category'})
        assert len(i['aliases']) == len({(a['locale'],normalize(a['locale'],a['name'])) for a in i['aliases']}), 'Repeated normalized alias on one identity'
        concept = normalize('en', re.sub(r'\([^)]*\)', '', i['names'].get('en',''))).strip()
        concepts[concept].append(key)
        for locale, label in i['names'].items():
            first = next((c for c in label if c.isalpha()), '')
            if first and first.islower(): warnings.append({'key':key,'locale':locale,'problem':'Initial capitalization needs review'})
        if 'Greek' in i['names'].get('tr','') or 'paste' in i['names'].get('tr','').lower():
            warnings.append({'key':key,'problem':'Possible untranslated/literal culinary terminology'})
        for locale, text in list(i['names'].items()) + [(a['locale'], a['name']) for a in i['aliases']]:
            if not text.strip() or text != text.strip() or text != unicodedata.normalize('NFC', text):
                errors.append({'key': key, 'locale': locale, 'problem': 'Empty/whitespace/non-NFC label'})
            owners[(locale, normalize(locale, text))].add(key)
        if not all(i['names'].get(locale) for locale in ('tr', 'en')):
            errors.append({'key': key, 'problem': 'Missing bilingual label'})
        if not i['provenance'] or set(i['categories']) - category_keys:
            errors.append({'key': key, 'problem': 'Missing provenance or category'})
        if i['observations']:
            errors.append({'key': key, 'problem': 'Unexpected empirical metadata in identity-only batch'})
        if any(p['sourceId'] == 'usda-sr-legacy' for p in i['provenance']) == key.startswith('project-'):
            errors.append({'key': key, 'problem': 'Incorrect source identity ownership'})
        if any(word in i['names']['en'].lower() for word in ('powder', 'flour')) and i['preferredUnit'] != 'g':
            errors.append({'key': key, 'problem': 'Powder default is not mass'})
    assert all(len(keys)==1 for keys in source_owners.values()), 'External source identity assigned to multiple canonical keys'
    collisions = [{'locale': locale, 'name': name, 'keys': sorted(keys)} for (locale, name), keys in sorted(owners.items()) if len(keys) > 1]
    queue = json.loads((OUT / 'localization-review.json').read_text())
    for item in queue:
        assert item['key'] in entries, 'Review item references missing identity'
        item['id'] = entries[item['key']]['id']
        item['tr'] = entries[item['key']]['names']['tr']
        item['en'] = entries[item['key']]['names']['en']
    report = dict(version=manifest['version'], capitalizationAndTranslationWarnings=warnings, conceptReviewCandidates=[dict(concept=c,keys=sorted(keys),meaning='Same English base label; qualified culinary forms may be distinct. Not automatically merged.') for c,keys in sorted(concepts.items()) if len(keys)>1], previousIdentities=len(old['ingredients']), auditedIdentities=len(entries), newlyAddedIdentities=len(entries)-len(old['ingredients']), stableHistoricalIds=len(old['ingredients']), structuralErrors=errors, aliasCollisionCandidates=collisions, linguisticReviewCount=len(queue), independentlyHumanReviewed=0, reviewBoundary='Structural and project semantic audit; human culinary terminology review remains open.', selectionExclusions=[dict(externalId='175186', reason='Black turtle bean mapping needs comparison with existing black bean identity; not added.'),dict(reference='Van otlu peyniri', reason='Excluded at v3 due unavailable primary page; reconsider only after current primary verification.')])
    if errors or collisions:
        raise ValueError(json.dumps(report, ensure_ascii=False))
    md = ['# Ingredient localization review queue', '', f'All {len(entries)} production identities were structurally audited; {len(queue)} terminology items require human review. No independent human review occurred.', '', 'These are naming refinements, not missing or invented source identities. Qualified forms remain separate. Review changes through a new catalog version; never reassign IDs.', '', '| Stable key | Turkish label | English label | Review question |', '| --- | --- | --- | --- |']
    for item in sorted(queue, key=lambda x: x['key']):
        md.append(f"| {item['key']} | {item['tr']} | {item['en']} | {item['reason']} |")
    artifacts = {OUT / 'quality-audit.json': json.dumps(report, ensure_ascii=False, indent=2)+'\n', ROOT / 'docs/INGREDIENT_LOCALIZATION_REVIEW.md': '\n'.join(md)+'\n'}
    for path, text in artifacts.items():
        if '--check' in sys.argv:
            assert path.read_text() == text, f'Stale audit artifact: {path}'
        else:
            path.write_text(text)
    print(f"Audited {len(entries)} identities; historical IDs intact; {len(queue)} human review items; no structural errors or exact alias collisions.")


if __name__ == '__main__':
    audit()
