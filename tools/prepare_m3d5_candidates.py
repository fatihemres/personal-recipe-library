#!/usr/bin/env python3
"""Screen M3D-5 beverage decisions against pinned CC0 evidence and Release 10."""
import argparse
import csv
import hashlib
import io
import json
import unicodedata
import zipfile
from collections import Counter
from pathlib import Path

from prepare_validation_catalog import encoded

ROOT = Path(__file__).resolve().parents[1]
PREP = ROOT / 'catalog/preparation/m3d5'
BASE = ROOT / 'catalog/releases/10'
SR_ARCHIVE_SHA = 'b80817294b8850530aaedf2e515c02593b1824f763a0ff356e5c2081643e6fd0'
BASE_MANIFEST_SHA = 'c909568c3d4a3487bde706042ba303455524f816dc620d64e5eed32c5193df87'


def load(path):
    return json.loads(path.read_text(encoding='utf-8'))


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def normalize(locale, value):
    value = unicodedata.normalize('NFC', value)
    if locale == 'tr':
        value = value.replace('I', 'ı').replace('İ', 'i')
    return ' '.join(value.lower().split())


def sr_facts(archive):
    assert digest(archive.read_bytes()) == SR_ARCHIVE_SHA, 'Pinned USDA SR archive changed'
    with zipfile.ZipFile(archive) as handle:
        with io.TextIOWrapper(handle.open('FoodData_Central_sr_legacy_food_csv_2018-04/food.csv'),
                              encoding='utf-8-sig') as stream:
            return {row['fdc_id']: row['description'] for row in csv.DictReader(stream)}


def wiki_facts():
    index = load(PREP / 'wikidata-index.json')
    facts = {}
    for entry in index:
        qid = entry['qid']
        raw = (PREP / 'wikidata-snapshots' / f'{qid}.json').read_bytes()
        assert digest(raw) == entry['sha256'], f'Wikidata snapshot changed: {qid}'
        entity = json.loads(raw)['entities'][qid]
        assert entity['id'] == qid and entity['lastrevid'] == entry['revision']
        facts[qid] = {
            'qid': qid, 'revision': entry['revision'], 'url': entry['url'],
            'labels': {locale: entity.get('labels', {}).get(locale, {}).get('value') for locale in ('tr', 'en')},
            'descriptions': {locale: entity.get('descriptions', {}).get(locale, {}).get('value') for locale in ('tr', 'en')},
            'snapshotSha256': entry['sha256'],
        }
    return facts


def build(sr):
    assert digest((BASE / 'manifest.json').read_bytes()) == BASE_MANIFEST_SHA, 'Release 10 manifest changed'
    baseline = load(BASE / 'ingredients.json')
    assert len(baseline['ingredients']) == 795 and len(baseline['categories']) == 50
    categories = {item['key'] for item in baseline['categories']}
    baseline_keys = {item['key'] for item in baseline['ingredients']}
    owners = {}
    source_owners = {}
    for item in baseline['ingredients']:
        for locale, name in item['names'].items():
            owners.setdefault((locale, normalize(locale, name)), set()).add(item['key'])
        for alias in item['aliases']:
            owners.setdefault((alias['locale'], normalize(alias['locale'], alias['name'])), set()).add(item['key'])
        for p in item['provenance']:
            source_owners[(p['sourceId'].split('-m3d')[0], p['externalId'])] = item['key']
    wiki = wiki_facts()
    with (PREP / 'selection.tsv').open(encoding='utf-8', newline='') as stream:
        rows = list(csv.DictReader(stream, delimiter='\t'))
    assert len({(row['source'], row['id']) for row in rows}) == len(rows)
    assert {row['source'] for row in rows} == {'sr', 'wikidata'}
    selected_labels = {}
    candidates, sr_records, wiki_records = [], [], []
    for row in rows:
        source, source_id, decision = row['source'], row['id'], row['decision']
        assert decision in ('ACCEPT', 'EXISTING', 'DEFER', 'REJECT') and row['reason']
        if source == 'sr':
            description = sr[source_id]
            source_url = f'https://fdc.nal.usda.gov/food-details/{source_id}/nutrients'
            evidence = dict(externalId=source_id, description=description)
            sr_records.append(dict(fdcId=source_id, description=description, sourceUrl=source_url))
            key = f'usda-sr-{source_id}'
        else:
            fact = wiki[source_id]
            description = json.dumps({k: fact[k] for k in ('qid', 'revision', 'labels', 'descriptions')},
                                     ensure_ascii=False, sort_keys=True, separators=(',', ':'))
            source_url = fact['url']
            evidence = dict(externalId=source_id, description=description)
            wiki_records.append(fact)
            key = f'wikidata-{source_id.lower()}'
        cats = row['categories'].split(';') if row['categories'] else []
        collision_signals = sorted({owner for locale, name in (('tr', row['tr']), ('en', row['en'])) if name
                                    for owner in owners.get((locale, normalize(locale, name)), set())})
        if decision == 'ACCEPT':
            assert row['tr'] and row['en'] and row['unit'] in ('g', 'mL') and cats
            assert set(cats) <= categories and key not in baseline_keys, f'Invalid category or existing key: {key} {cats}'
            assert not collision_signals, f'Catalog label collision: {source_id}: {collision_signals}'
            assert not any(external == source_id and (name.startswith('usda-sr') if source == 'sr' else name.startswith('wikidata'))
                           for (name, external) in source_owners), f'Source ID already owned: {source_id}'
            for locale, name in (('tr', row['tr']), ('en', row['en'])):
                assert name == name.strip() and name == unicodedata.normalize('NFC', name)
                label_key = locale, normalize(locale, name)
                assert label_key not in selected_labels, f'Duplicate selected label: {name}'
                selected_labels[label_key] = source_id
        else:
            assert not any(row[field] for field in ('tr', 'en', 'categories', 'unit'))
        candidates.append(dict(candidateId=f'm3d5-{source}-{source_id}', source=source,
                               sourceIdentifier=source_id, sourceDescription=description,
                               sourceUrl=source_url, sourceLicense='CC0-1.0', sourceEvidence=evidence,
                               proposedTurkish=row['tr'] or None, proposedEnglish=row['en'] or None,
                               categories=cats, preferredUnit=row['unit'] or None,
                               decision=decision, reason=row['reason'], collisionSignals=collision_signals,
                               humanLinguisticApproval=False, observations=[]))
    candidates.sort(key=lambda c: c['candidateId'])
    sr_records.sort(key=lambda r: int(r['fdcId']))
    wiki_records.sort(key=lambda r: r['qid'])
    outputs = {
        PREP / 'candidates.json': encoded(candidates),
        PREP / 'sr-records.json': encoded(sr_records),
        PREP / 'wikidata-records.json': encoded(wiki_records),
    }
    manifest = dict(baselineRelease=10, baselineManifestSha256=BASE_MANIFEST_SHA,
                    srArchiveSha256=SR_ARCHIVE_SHA,
                    wikidataIndexSha256=digest((PREP / 'wikidata-index.json').read_bytes()),
                    selectionSha256=digest((PREP / 'selection.tsv').read_bytes()),
                    decisions=dict(Counter(c['decision'] for c in candidates)),
                    sourceCounts=dict(Counter(c['source'] for c in candidates)),
                    candidateCount=len(candidates),
                    outputSha256={path.name: digest(raw) for path, raw in outputs.items()})
    outputs[PREP / 'manifest.json'] = encoded(manifest)
    return outputs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    sr = sr_facts(args.archive) if args.archive else {
        row['fdcId']: row['description'] for row in load(PREP / 'sr-records.json')}
    outputs = build(sr)
    for path, raw in outputs.items():
        if args.check:
            assert path.read_bytes() == raw, f'Stale M3D-5 candidate artifact: {path}'
        else:
            path.write_bytes(raw)
    print('M3D-5 candidates:', json.loads(outputs[PREP / 'manifest.json'])['decisions'])


if __name__ == '__main__':
    main()
