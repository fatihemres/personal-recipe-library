#!/usr/bin/env python3
"""Release-9 source, decision, identity and reproducibility boundaries."""
import hashlib
import json
import unittest
from collections import Counter

import prepare_m3d3_candidates as preparation
import prepare_m3d3_catalog as release
from prepare_validation_catalog import stable


class ReleaseNineTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.previous = release.load(release.OLD / 'ingredients.json')
        cls.current = release.load(release.OUT / 'ingredients.json')
        cls.candidates = release.load(release.PREP / 'candidates.json')
        cls.manifest = release.load(release.OUT / 'manifest.json')

    def test_all_candidates_have_auditable_source_and_disposition(self):
        self.assertEqual(len(self.candidates), 131)
        self.assertEqual(len({c['candidateId'] for c in self.candidates}), 131)
        self.assertEqual(Counter(c['decision'] for c in self.candidates),
                         {'ACCEPT': 98, 'DEFER': 29, 'EXISTING': 3, 'REJECT': 1})
        self.assertTrue(all(c['sourceUrl'].startswith('https://fdc.nal.usda.gov/')
                            and c['reason'] and not c['humanLinguisticApproval']
                            and c['sourceLicense'] == 'CC0-1.0' for c in self.candidates))
        self.assertEqual(preparation.build({r['fdc_id']: r for r in release.load(release.PREP/'source-records.json')})[release.PREP/'candidates.json'],
                         (release.PREP/'candidates.json').read_bytes())

    def test_existing_647_objects_and_immutable_source_versions(self):
        old = {i['key']: i for i in self.previous['ingredients']}
        now = {i['key']: i for i in self.current['ingredients']}
        self.assertEqual(len(old), 647)
        self.assertEqual(len(now), 745)
        self.assertEqual(self.previous['categories'], self.current['categories'])
        for key, ingredient in old.items():
            self.assertEqual(now[key], ingredient)
        additions = [i for key, i in now.items() if key not in old]
        self.assertEqual(len(additions), 98)
        self.assertEqual({i['key'] for i in additions},
                         {'usda-sr-'+c['sourceIdentifier'] for c in self.candidates if c['decision']=='ACCEPT'})
        for ingredient in additions:
            self.assertEqual(ingredient['id'], stable('ingredient', ingredient['key']))
            self.assertEqual(ingredient['aliases'], [])
            self.assertEqual(ingredient['observations'], [])
            self.assertEqual({p['sourceId'] for p in ingredient['provenance']},
                             {release.USDA_SOURCE, release.CURATION_SOURCE})
        self.assertEqual(sum(len(i['aliases']) for i in now.values()), 269)

    def test_release9_is_offline_reproducible_and_allowlisted(self):
        self.assertEqual(self.manifest['version'], 9)
        self.assertEqual({s['id'] for s in self.manifest['sources'][-2:]},
                         {release.USDA_SOURCE, release.CURATION_SOURCE})
        registry = {s['id']: s for s in release.load(release.ROOT/'catalog/sources.json')['sources']}
        for source in self.manifest['sources']:
            self.assertTrue(registry[source['id']]['seedApproved'])
            self.assertEqual(source['license'], registry[source['id']]['license'])
            artifact = self.manifest['evidence'][source['id']]
            self.assertEqual(hashlib.sha256((release.OUT/artifact['file']).read_bytes()).hexdigest(), artifact['sha256'])
        self.assertFalse(registry['foodon']['seedApproved'])
        for path, expected in release.build().items():
            self.assertEqual(path.read_bytes(), expected, path)


if __name__ == '__main__':
    unittest.main()
