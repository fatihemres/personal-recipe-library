#!/usr/bin/env python3
"""Deterministic audit of all production labels/identities, not linguistic certification.
Run normally to generate reports, or --check to require committed report freshness.
"""
import collections
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
    old = json.loads((ROOT / 'catalog/releases/2/ingredients.json').read_text())
    entries = {i['key']: i for i in data['ingredients']}
    assert len(entries) == len(data['ingredients']), 'Duplicate canonical keys'
    assert len({i['id'] for i in entries.values()}) == len(entries), 'Duplicate stable IDs'
    for i in old['ingredients']:
        assert i['key'] in entries and entries[i['key']]['id'] == i['id'], 'Lost historical identity'
    category_keys = {c['key'] for c in data['categories']}
    owners = collections.defaultdict(set)
    errors = []
    for key, i in entries.items():
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
    collisions = [{'locale': locale, 'name': name, 'keys': sorted(keys)} for (locale, name), keys in sorted(owners.items()) if len(keys) > 1]
    queue = json.loads((OUT / 'localization-review.json').read_text())
    for item in queue:
        assert item['key'] in entries, 'Review item references missing identity'
        item['id'] = entries[item['key']]['id']
        item['tr'] = entries[item['key']]['names']['tr']
        item['en'] = entries[item['key']]['names']['en']
    report = dict(version=3, previousIdentities=len(old['ingredients']), auditedIdentities=len(entries), newlyAddedIdentities=len(entries)-len(old['ingredients']), stableHistoricalIds=len(old['ingredients']), structuralErrors=errors, aliasCollisionCandidates=collisions, linguisticReviewCount=len(queue), independentlyHumanReviewed=0, reviewBoundary='Structural and project semantic audit; human culinary terminology review remains open.', selectionExclusions=[dict(externalId='175186', reason='Black turtle bean mapping needs comparison with existing black bean identity; not added.'),dict(reference='Van otlu peyniri', reason='Primary page unavailable during current verification; excluded from this batch, not a licensed raw import.')])
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
