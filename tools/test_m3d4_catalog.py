#!/usr/bin/env python3
"""Release-10 candidate, provenance, decision and identity invariants."""
import hashlib
import json
import unittest
from collections import Counter

import prepare_m3d4_candidates as preparation
import prepare_m3d4_catalog as release
from prepare_validation_catalog import stable


class ReleaseTenTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.old = release.load(release.OLD / 'ingredients.json')
        cls.current = release.load(release.OUT / 'ingredients.json')
        cls.candidates = release.load(release.PREP / 'candidates.json')
        cls.manifest = release.load(release.OUT / 'manifest.json')

    def test_candidate_ledger_is_complete_and_reproducible_offline(self):
        self.assertEqual(len(self.candidates), 65)
        self.assertEqual(len({c['candidateId'] for c in self.candidates}), 65)
        self.assertEqual(Counter(c['decision'] for c in self.candidates),
                         {'ACCEPT': 50, 'DEFER': 9, 'EXISTING': 4, 'REJECT': 2})
        self.assertTrue(all(c['sourceUrl'].startswith('https://') and c['reason']
                            and not c['humanLinguisticApproval'] and not c['observations']
                            and c['sourceLicense'] == 'CC0-1.0' for c in self.candidates))
        facts = {r['fdcId']: r for r in release.load(release.PREP / 'foundation-records.json')}
        for path, content in preparation.build(facts).items():
            self.assertEqual(path.read_bytes(), content, path)
        for item in release.load(release.PREP / 'wikidata-index.json'):
            raw = (release.PREP / 'wikidata-snapshots' / f"{item['qid']}.json").read_bytes()
            self.assertEqual(hashlib.sha256(raw).hexdigest(), item['sha256'])
            entity = json.loads(raw)['entities'][item['qid']]
            self.assertEqual(entity['lastrevid'], item['revision'])

    def test_all_745_objects_stable_and_categories_unchanged(self):
        old = {i['key']: i for i in self.old['ingredients']}
        now = {i['key']: i for i in self.current['ingredients']}
        self.assertEqual(len(old), 745)
        self.assertEqual(len(now), 795)
        self.assertEqual(self.old['categories'], self.current['categories'])
        self.assertTrue(all(now[key] == value for key, value in old.items()))
        additions = [item for key, item in now.items() if key not in old]
        self.assertEqual(len(additions), 50)
        self.assertEqual(sum(i['ingredientType'] == 'beverage' for i in additions), 8)
        self.assertEqual(sum('turkish-regional' in i['categories'] for i in additions), 11)
        for item in additions:
            self.assertEqual(item['id'], stable('ingredient', item['key']))
            self.assertEqual(item['aliases'], [])
            self.assertEqual(item['observations'], [])
            self.assertEqual(len(item['provenance']), 2)
        self.assertEqual(sum(len(i['aliases']) for i in now.values()), 269)

    def test_source_versions_and_historical_snapshot_are_immutable(self):
        self.assertEqual(self.manifest['version'], 10)
        self.assertEqual({s['id'] for s in self.manifest['sources'][-3:]},
                         {release.FOUNDATION_SOURCE, release.WIKIDATA_SOURCE, release.CURATION_SOURCE})
        registry = {s['id']: s for s in release.load(release.ROOT / 'catalog/sources.json')['sources']}
        for source in self.manifest['sources']:
            self.assertTrue(registry[source['id']]['seedApproved'])
            self.assertEqual(source['license'], registry[source['id']]['license'])
            artifact = self.manifest['evidence'][source['id']]
            self.assertEqual(hashlib.sha256((release.OUT / artifact['file']).read_bytes()).hexdigest(), artifact['sha256'])
        self.assertFalse(registry['foodon']['seedApproved'])
        self.assertEqual(hashlib.sha256((release.OLD / 'manifest.json').read_bytes()).hexdigest(),
                         preparation.BASE_MANIFEST_SHA)
        for path, expected in release.build().items():
            self.assertEqual(path.read_bytes(), expected, path)

    def test_ambiguous_items_not_silently_added(self):
        excluded = {c['sourceIdentifier']: c['decision'] for c in self.candidates if c['decision'] != 'ACCEPT'}
        self.assertEqual(excluded['Q1817099'], 'REJECT')
        self.assertEqual(excluded['Q1866347'], 'DEFER')
        self.assertEqual(excluded['2262073'], 'DEFER')
        self.assertEqual(excluded['Q235169'], 'EXISTING')
        keys = {i['key'] for i in self.current['ingredients']}
        self.assertNotIn('wikidata-q1866347', keys)
        self.assertNotIn('usda-foundation-2262073', keys)


if __name__ == '__main__':
    unittest.main()
