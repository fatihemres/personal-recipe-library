#!/usr/bin/env python3
"""Release-11 beverage provenance, screening, and historical-identity gates."""
import hashlib
import json
import unittest
from collections import Counter

import prepare_m3d5_candidates as preparation
import prepare_m3d5_catalog as release
from prepare_validation_catalog import stable


class ReleaseElevenTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.old = release.load(release.OLD / 'ingredients.json')
        cls.current = release.load(release.OUT / 'ingredients.json')
        cls.candidates = release.load(release.PREP / 'candidates.json')
        cls.manifest = release.load(release.OUT / 'manifest.json')

    def test_pinned_candidate_ledger_reproducible_offline(self):
        self.assertEqual(len(self.candidates), 85)
        self.assertEqual(len({c['candidateId'] for c in self.candidates}), 85)
        self.assertEqual(Counter(c['decision'] for c in self.candidates),
                         {'ACCEPT': 59, 'DEFER': 16, 'EXISTING': 10})
        self.assertTrue(all(c['sourceUrl'].startswith('https://') and c['reason']
                            and not c['humanLinguisticApproval'] and not c['observations']
                            and c['sourceLicense'] == 'CC0-1.0' for c in self.candidates))
        sr = {r['fdcId']: r['description'] for r in release.load(release.PREP / 'sr-records.json')}
        for path, content in preparation.build(sr).items():
            self.assertEqual(path.read_bytes(), content, path)
        for entry in release.load(release.PREP / 'wikidata-index.json'):
            raw = (release.PREP / 'wikidata-snapshots' / f"{entry['qid']}.json").read_bytes()
            self.assertEqual(hashlib.sha256(raw).hexdigest(), entry['sha256'])
            entity = json.loads(raw)['entities'][entry['qid']]
            self.assertEqual(entity['lastrevid'], entry['revision'])

    def test_release10_objects_and_categories_are_unchanged(self):
        old = {i['key']: i for i in self.old['ingredients']}
        now = {i['key']: i for i in self.current['ingredients']}
        self.assertEqual(len(old), 795)
        self.assertEqual(len(now), 854)
        self.assertEqual(self.old['categories'], self.current['categories'])
        self.assertTrue(all(now[key] == value for key, value in old.items()))
        additions = [item for key, item in now.items() if key not in old]
        self.assertEqual(len(additions), 59)
        self.assertEqual(sum(i['ingredientType'] == 'beverage' for i in additions), 59)
        self.assertEqual(sum(i['ingredientType'] == 'beverage' for i in now.values()), 188)
        for item in additions:
            self.assertEqual(item['id'], stable('ingredient', item['key']))
            self.assertEqual(item['aliases'], [])
            self.assertEqual(item['observations'], [])
            self.assertEqual(len(item['provenance']), 2)
        self.assertEqual(sum(len(i['aliases']) for i in now.values()), 269)

    def test_new_source_versions_and_package_are_immutable(self):
        self.assertEqual(self.manifest['version'], 11)
        self.assertEqual({s['id'] for s in self.manifest['sources'][-3:]},
                         {release.SR_SOURCE, release.WIKI_SOURCE, release.CURATION_SOURCE})
        registry = {s['id']: s for s in release.load(release.ROOT / 'catalog/sources.json')['sources']}
        for source in self.manifest['sources']:
            self.assertTrue(registry[source['id']]['seedApproved'])
            self.assertEqual(source['license'], registry[source['id']]['license'])
            artifact = self.manifest['evidence'][source['id']]
            self.assertEqual(hashlib.sha256((release.OUT / artifact['file']).read_bytes()).hexdigest(),
                             artifact['sha256'])
        self.assertFalse(registry['foodon']['seedApproved'])
        self.assertEqual(hashlib.sha256((release.OLD / 'manifest.json').read_bytes()).hexdigest(),
                         preparation.BASE_MANIFEST_SHA)
        for path, expected in release.build().items():
            self.assertEqual(path.read_bytes(), expected, path)

    def test_ambiguous_and_existing_forms_are_excluded(self):
        excluded = {c['sourceIdentifier']: c['decision'] for c in self.candidates if c['decision'] != 'ACCEPT'}
        for identifier, decision in [('Q192892', 'DEFER'), ('Q26877308', 'DEFER'),
                                     ('Q26877423', 'DEFER'), ('Q392001', 'EXISTING'),
                                     ('169110', 'EXISTING'), ('174863', 'DEFER')]:
            self.assertEqual(excluded[identifier], decision)
        keys = {i['key'] for i in self.current['ingredients']}
        self.assertNotIn('wikidata-q192892', keys)
        self.assertNotIn('usda-sr-174863', keys)
        self.assertIn('wikidata-q693187', keys)


if __name__ == '__main__':
    unittest.main()
