#!/usr/bin/env python3
"""Screen pinned Foundation/Wikidata evidence against immutable Release 9."""
import argparse
import csv
import hashlib
import json
import unicodedata
import zipfile
from collections import Counter
from pathlib import Path

from prepare_validation_catalog import encoded

ROOT = Path(__file__).resolve().parents[1]
PREP = ROOT / 'catalog/preparation/m3d4'
BASE = ROOT / 'catalog/releases/9'
ARCHIVE_SHA = '186e988ec542e913f51ef62b86a47758e8cdd0d1dc3889e7b055581f3c09c77a'
BASE_MANIFEST_SHA = '3f0da2052988e65c9c543340c50ce7c8ea36323d687b5925c88e1781631b7712'


def load(path):
    return json.loads(path.read_text(encoding='utf-8'))


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def normalize(locale, value):
    value = unicodedata.normalize('NFC', value)
    if locale == 'tr':
        value = value.replace('I', 'ı').replace('İ', 'i')
    return ' '.join(value.lower().split())


def foundation_facts(archive):
    raw = archive.read_bytes()
    assert digest(raw) == ARCHIVE_SHA, 'Foundation archive checksum changed'
    with zipfile.ZipFile(archive) as handle:
        members = handle.namelist()
        assert len(members) == 1 and members[0].endswith('.json')
        foods = json.loads(handle.read(members[0]))['FoundationFoods']
    facts = {str(food['fdcId']): food for food in foods if isinstance(food, dict)}
    assert len(facts) >= 360 and len(facts) == len(foods) - foods.count(None)
    return facts


def wikidata_facts():
    index = load(PREP / 'wikidata-index.json')
    facts = {}
    for entry in index:
        qid = entry['qid']
        raw = (PREP / 'wikidata-snapshots' / f'{qid}.json').read_bytes()
        assert digest(raw) == entry['sha256'], f'Wikidata snapshot changed: {qid}'
        entity = json.loads(raw)['entities'][qid]
        assert entity['id'] == qid and entity['lastrevid'] == entry['revision']
        facts[qid] = dict(qid=qid, revision=entry['revision'], sourceUrl=entry['url'],
                          labels={k: entity.get('labels', {}).get(k, {}).get('value') for k in ('tr', 'en')},
                          descriptions={k: entity.get('descriptions', {}).get(k, {}).get('value') for k in ('tr', 'en')},
                          snapshotSha256=entry['sha256'])
    return facts


def build(foundation):
    assert digest((BASE / 'manifest.json').read_bytes()) == BASE_MANIFEST_SHA, 'Release 9 manifest changed'
    baseline = load(BASE / 'ingredients.json')
    assert len(baseline['ingredients']) == 745 and len(baseline['categories']) == 50
    categories = {category['key'] for category in baseline['categories']}
    owners = {}
    source_owners = {}
    for item in baseline['ingredients']:
        for locale, name in item['names'].items():
            owners.setdefault((locale, normalize(locale, name)), set()).add(item['key'])
        for alias in item['aliases']:
            owners.setdefault((alias['locale'], normalize(alias['locale'], alias['name'])), set()).add(item['key'])
        for provenance in item['provenance']:
            source_owners[(provenance['sourceId'], provenance['externalId'])] = item['key']
    wiki = wikidata_facts()
    with (PREP / 'selection.tsv').open(encoding='utf-8', newline='') as stream:
        rows = list(csv.DictReader(stream, delimiter='\t'))
    assert len({(row['source'], row['id']) for row in rows}) == len(rows)
    assert {row['source'] for row in rows} == {'foundation', 'wikidata'}
    accepted_labels = {}
    candidates = []
    selected_foundation = []
    selected_wiki = []
    for row in rows:
        source, source_id, decision = row['source'], row['id'], row['decision']
        assert decision in ('ACCEPT', 'EXISTING', 'DEFER', 'REJECT') and row['reason']
        if source == 'foundation':
            fact = foundation[source_id]
            descriptor = fact['description']
            source_url = f'https://fdc.nal.usda.gov/food-details/{source_id}/nutrients'
            evidence = dict(externalId=source_id, description=descriptor)
            category = fact.get('foodCategory')
            selected_foundation.append(dict(fdcId=source_id, description=descriptor,
                                            foodCategory=category.get('description') if isinstance(category, dict) else category,
                                            sourceUrl=source_url))
            source_key = f'usda-foundation-{source_id}'
        else:
            fact = wiki[source_id]
            descriptor = json.dumps({k: fact[k] for k in ('qid', 'revision', 'labels', 'descriptions')},
                                    ensure_ascii=False, sort_keys=True, separators=(',', ':'))
            source_url = fact['sourceUrl']
            evidence = dict(externalId=source_id, description=descriptor)
            selected_wiki.append(fact)
            source_key = f'wikidata-{source_id.lower()}'
        cats = row['categories'].split(';') if row['categories'] else []
        collision_signals = sorted({key for label in (row['tr'], row['en']) if label
                                    for locale in ('tr', 'en')
                                    for key in owners.get((locale, normalize(locale, label)), set())})
        if decision == 'ACCEPT':
            assert row['tr'] and row['en'] and row['unit'] in ('g', 'mL') and cats
            assert set(cats) <= categories and not collision_signals, f"Collision: {source_id} {collision_signals}"
            for locale, label in (('tr', row['tr']), ('en', row['en'])):
                assert label == label.strip() and label == unicodedata.normalize('NFC', label)
                key = (locale, normalize(locale, label))
                assert key not in accepted_labels, f'Duplicate selected label: {label}'
                accepted_labels[key] = source_id
            assert source_key not in {item['key'] for item in baseline['ingredients']}
        else:
            assert not (row['tr'] or row['en'] or cats or row['unit']), f'Non-accepted candidate has catalog fields: {source_id}'
        candidates.append(dict(candidateId=f'm3d4-{source}-{source_id}', source=source,
                               sourceIdentifier=source_id, sourceDescription=descriptor,
                               sourceUrl=source_url, sourceLicense='CC0-1.0', sourceEvidence=evidence,
                               proposedTurkish=row['tr'] or None, proposedEnglish=row['en'] or None,
                               categories=cats, preferredUnit=row['unit'] or None,
                               decision=decision, reason=row['reason'], collisionSignals=collision_signals,
                               humanLinguisticApproval=False, observations=[]))
    candidates.sort(key=lambda c: c['candidateId'])
    selected_foundation.sort(key=lambda x: int(x['fdcId']))
    selected_wiki.sort(key=lambda x: x['qid'])
    outputs = {
        PREP / 'candidates.json': encoded(candidates),
        PREP / 'foundation-records.json': encoded(selected_foundation),
        PREP / 'wikidata-records.json': encoded(selected_wiki),
    }
    report = dict(baselineRelease=9, baselineManifestSha256=BASE_MANIFEST_SHA,
                  foundationArchiveSha256=ARCHIVE_SHA,
                  wikidataIndexSha256=digest((PREP / 'wikidata-index.json').read_bytes()),
                  selectionSha256=digest((PREP / 'selection.tsv').read_bytes()),
                  decisions=dict(Counter(c['decision'] for c in candidates)),
                  sourceCounts=dict(Counter(c['source'] for c in candidates)),
                  candidateCount=len(candidates),
                  outputSha256={path.name: digest(raw) for path, raw in outputs.items()})
    outputs[PREP / 'manifest.json'] = encoded(report)
    return outputs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, help='Pinned original USDA Foundation ZIP')
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if args.archive:
        facts = foundation_facts(args.archive)
    else:
        facts = {row['fdcId']: row for row in load(PREP / 'foundation-records.json')}
    outputs = build(facts)
    for path, raw in outputs.items():
        if args.check:
            assert path.read_bytes() == raw, f'Stale M3D-4 candidate artifact: {path}'
        else:
            path.write_bytes(raw)
    print('M3D-4 candidates:', load(PREP / 'manifest.json')['decisions'] if not args.check else json.loads(outputs[PREP / 'manifest.json'])['decisions'])


if __name__ == '__main__':
    main()
