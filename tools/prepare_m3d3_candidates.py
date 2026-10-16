#!/usr/bin/env python3
"""Pin and screen M3D-3 USDA candidates before touching production release 8.

Requires the exact 2018-04 archive for initial extraction. --check uses the
committed selection evidence and never downloads or reads a personal database.
"""
import argparse
import csv
import hashlib
import io
import json
import unicodedata
import zipfile
from collections import Counter
from pathlib import Path

from prepare_validation_catalog import ARCHIVE_HASH, encoded

ROOT = Path(__file__).resolve().parents[1]
PREP = ROOT / 'catalog/preparation/m3d3'
BASE = ROOT / 'catalog/releases/8'
BASE_SHA = '9bad5c51370bb6d13c2bbd313b3279244a58b8d57e1a720be1a9d4e75f213e65'


def load(path):
    return json.loads(path.read_text(encoding='utf-8'))


def normalize(locale, value):
    value = unicodedata.normalize('NFC', value)
    if locale == 'tr':
        value = value.replace('I', 'ı').replace('İ', 'i')
    return ' '.join(value.lower().split())


def rows(path):
    with path.open(encoding='utf-8', newline='') as stream:
        return list(csv.DictReader(stream, delimiter='\t'))


def archive_facts(path):
    raw = path.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == ARCHIVE_HASH, 'Pinned USDA archive checksum changed'
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        name = next(n for n in archive.namelist() if n.endswith('/food.csv'))
        with io.TextIOWrapper(archive.open(name), encoding='utf-8-sig') as stream:
            return {row['fdc_id']: row for row in csv.DictReader(stream)}


def build(facts):
    assert hashlib.sha256((BASE / 'manifest.json').read_bytes()).hexdigest() == BASE_SHA, 'Release 8 baseline manifest changed'
    baseline = load(BASE / 'ingredients.json')
    assert len(baseline['ingredients']) == 647 and len(baseline['categories']) == 50
    allowed_categories = {c['key'] for c in baseline['categories']}
    owners = {}
    source_owners = {}
    for item in baseline['ingredients']:
        for locale, name in item['names'].items():
            owners.setdefault((locale, normalize(locale, name)), set()).add(item['key'])
        for alias in item['aliases']:
            owners.setdefault((alias['locale'], normalize(alias['locale'], alias['name'])), set()).add(item['key'])
        for provenance in item['provenance']:
            if provenance['sourceId'].startswith('usda-sr-legacy'):
                source_owners[provenance['externalId']] = item['key']
    accepted = rows(PREP / 'selection.tsv')
    other = rows(PREP / 'other-decisions.tsv')
    prior = [d for d in load(BASE / 'm3d2b-decisions.json') if d['source'] == 'usda' and d['decision'] == 'DEFER']
    assert len(prior) == 27
    selected_ids = [r['fdc_id'] for r in accepted + other]
    prior_ids = [d['sourceIdentifier'] for d in prior]
    assert len(set(selected_ids + prior_ids)) == len(selected_ids) + len(prior_ids), 'Repeated candidate source ID'
    assert set(selected_ids + prior_ids) <= facts.keys(), 'Missing USDA source descriptor'
    accepted_owners = {}
    candidates = []
    for row in accepted:
        source_id = row['fdc_id']
        categories = row['categories'].split(';')
        assert categories and set(categories) <= allowed_categories and len(categories) == len(set(categories))
        assert source_id not in source_owners, f'Already mapped source: {source_id}'
        assert row['unit'] in ('g', 'mL') and row['decision_reason']
        for locale, label in (('tr', row['tr']), ('en', row['en'])):
            assert label and label == label.strip() and label == unicodedata.normalize('NFC', label)
            normalized = (locale, normalize(locale, label))
            assert normalized not in owners and normalized not in accepted_owners, f'Name collision: {label}'
            accepted_owners[normalized] = source_id
        fact = facts[source_id]
        assert fact['data_type'] == 'sr_legacy_food'
        ingredient_type = 'beverage' if fact['food_category_id'] == '14' or categories[0] in ('coffee', 'tea', 'juices', 'mixers') else 'food'
        candidates.append(dict(candidateId=f'm3d3-usda-{source_id}', sourceIdentifier=source_id,
            sourceDescription=fact['description'], sourceUrl=f'https://fdc.nal.usda.gov/food-details/{source_id}/nutrients',
            sourceCategoryId=fact['food_category_id'], proposedTurkish=row['tr'], proposedEnglish=row['en'],
            categories=categories, preferredUnit=row['unit'], ingredientType=ingredient_type,
            decision='ACCEPT', reason=row['decision_reason'], humanLinguisticApproval=False,
            sourceLicense='CC0-1.0', observations=[], collisionSignals=[]))
    for row in other:
        source_id = row['fdc_id']
        assert row['decision'] in ('EXISTING', 'DEFER', 'REJECT') and row['reason']
        assert (source_id in source_owners) == (row['decision'] == 'EXISTING')
        fact = facts[source_id]
        candidates.append(dict(candidateId=f'm3d3-usda-{source_id}', sourceIdentifier=source_id,
            sourceDescription=fact['description'], sourceUrl=f'https://fdc.nal.usda.gov/food-details/{source_id}/nutrients',
            sourceCategoryId=fact['food_category_id'], proposedTurkish=None, proposedEnglish=None,
            categories=[], preferredUnit=None, ingredientType=None, decision=row['decision'],
            reason=row['reason'], humanLinguisticApproval=False, sourceLicense='CC0-1.0',
            observations=[], collisionSignals=[source_owners[source_id]] if source_id in source_owners else []))
    for decision in prior:
        source_id = decision['sourceIdentifier']
        fact = facts[source_id]
        candidates.append(dict(candidateId=f'm3d3-usda-{source_id}', sourceIdentifier=source_id,
            sourceDescription=fact['description'], sourceUrl=f'https://fdc.nal.usda.gov/food-details/{source_id}/nutrients',
            sourceCategoryId=fact['food_category_id'], proposedTurkish=None, proposedEnglish=None,
            categories=[], preferredUnit=None, ingredientType=None, decision='DEFER',
            reason=decision['reason'], humanLinguisticApproval=False, sourceLicense='CC0-1.0',
            observations=[], collisionSignals=decision['collisionSignals']))
    candidates.sort(key=lambda c: c['sourceIdentifier'])
    evidence = [facts[c['sourceIdentifier']] for c in candidates]
    manifest = dict(baselineRelease=8, baselineManifestSha256=BASE_SHA,
        archiveSha256=ARCHIVE_HASH, source='USDA FoodData Central SR Legacy 2018-04',
        selectionSha256=hashlib.sha256((PREP / 'selection.tsv').read_bytes()).hexdigest(),
        otherDecisionsSha256=hashlib.sha256((PREP / 'other-decisions.tsv').read_bytes()).hexdigest(),
        candidates=len(candidates), decisions=dict(Counter(c['decision'] for c in candidates)),
        sourceRecordsSha256=hashlib.sha256(encoded(evidence)).hexdigest(),
        candidatesSha256=hashlib.sha256(encoded(candidates)).hexdigest())
    return {PREP / 'source-records.json': encoded(evidence), PREP / 'candidates.json': encoded(candidates), PREP / 'manifest.json': encoded(manifest)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    assert args.archive or args.check, 'Supply --archive for source extraction or --check for offline verification'
    if args.archive:
        facts = archive_facts(args.archive)
    else:
        evidence = load(PREP / 'source-records.json')
        facts = {row['fdc_id']: row for row in evidence}
    outputs = build(facts)
    if args.check:
        for path, content in outputs.items():
            assert path.read_bytes() == content, f'Stale candidate artifact: {path}'
    else:
        for path, content in outputs.items():
            path.write_bytes(content)
    manifest = json.loads(outputs[PREP / 'manifest.json'])
    print('M3D-3 candidate decisions:', manifest['decisions'])


if __name__ == '__main__':
    main()
