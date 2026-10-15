#!/usr/bin/env python3
"""Release-8 reproducibility, rights and identity boundary tests."""
import hashlib
import json
from collections import Counter
from pathlib import Path
import shutil
import tempfile
import unittest

import prepare_m3d2b_catalog as release
from prepare_validation_catalog import stable


class ReleaseEightTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.old = json.loads((release.OLD/'ingredients.json').read_text())
        cls.current = json.loads((release.OUT/'ingredients.json').read_text())
        cls.decisions = json.loads((release.OUT/'m3d2b-decisions.json').read_text())
        cls.manifest = json.loads((release.OUT/'manifest.json').read_text())

    def test_every_candidate_has_one_auditable_decision(self):
        self.assertEqual(len(self.decisions), 194)
        self.assertEqual(len({d['candidateId'] for d in self.decisions}), 194)
        self.assertEqual(Counter(d['decision'] for d in self.decisions if d['source']=='usda'), {'ACCEPT':79,'DEFER':27})
        self.assertEqual(Counter(d['decision'] for d in self.decisions if d['source']=='foodon'), {'DEFER':84,'EXISTING':3,'REJECT':1})
        self.assertTrue(all(d['sourceUrl'].startswith(('https://','http://')) and d['reason'] and not d['independentlyHumanApproved'] for d in self.decisions))
        self.assertFalse(any(d['source']=='foodon' and d['decision']=='ACCEPT' for d in self.decisions))

    def test_immutable_release7_stable_ids_and_exact_new_source_facts(self):
        old = {i['key']:i for i in self.old['ingredients']}
        new = {i['key']:i for i in self.current['ingredients']}
        self.assertEqual(self.old['categories'],self.current['categories'])
        self.assertEqual(len(old),568)
        self.assertEqual(len(new),647)
        self.assertTrue(all(new[k]==v for k,v in old.items()))
        accepted = {d['sourceIdentifier'] for d in self.decisions if d['decision']=='ACCEPT'}
        added = [i for k,i in new.items() if k not in old]
        self.assertEqual({i['key'] for i in added},{'usda-sr-'+external for external in accepted})
        evidence = {e['externalId']:e['description'] for e in json.loads((release.OUT/'m3d2b-usda-records.json').read_text())}
        prepared = {(e['source'],e['id']):e for e in json.loads((release.PREP/'source-records.json').read_text())}
        for item in added:
            external=item['key'].removeprefix('usda-sr-')
            self.assertEqual(item['id'],stable('ingredient',item['key']))
            self.assertEqual(evidence[external],prepared['usda',external]['label'])
            self.assertEqual({p['sourceId'] for p in item['provenance']},{release.USDA_SOURCE,release.CURATION_SOURCE})
            self.assertEqual(item['observations'],[])
            self.assertEqual(item['aliases'],[])
            self.assertEqual(item['preferredUnit'],'mL' if item['dimensions']==['volume'] else 'g')
            self.assertTrue(item['names']['tr'] and item['names']['en'])
        self.assertEqual(sum(len(i['aliases']) for i in new.values()),269)

    def test_release8_sources_are_pinned_and_foodon_is_not_allowlisted(self):
        self.assertEqual(self.manifest['version'],8)
        self.assertEqual(len(self.manifest['sources']),8)
        self.assertEqual({s['id'] for s in self.manifest['sources'][-2:]},{release.USDA_SOURCE,release.CURATION_SOURCE})
        self.assertFalse(any(s['id']=='foodon' for s in self.manifest['sources']))
        registry={s['id']:s for s in json.loads((release.ROOT/'catalog/sources.json').read_text())['sources']}
        self.assertFalse(registry['foodon']['seedApproved'])
        for source in self.manifest['sources']:
            descriptor=self.manifest['evidence'][source['id']]
            self.assertEqual(source['artifactSha256'],descriptor['sha256'])
            self.assertEqual(hashlib.sha256((release.OUT/descriptor['file']).read_bytes()).hexdigest(),descriptor['sha256'])

    def test_offline_regeneration_and_pinned_candidate_evidence(self):
        for path,expected in release.build().items():
            self.assertEqual(path.read_bytes(),expected,path)
        self.assertEqual(hashlib.sha256((release.OLD/'manifest.json').read_bytes()).hexdigest(),release.MANIFEST_SHA)

    def test_corrupt_candidate_source_and_name_collision_fail_before_writes(self):
        with tempfile.TemporaryDirectory() as name:
            root=Path(name)
            shutil.copytree(release.OLD,root/'catalog/releases/7')
            shutil.copytree(release.PREP,root/'catalog/preparation/m3d2a')
            (root/'catalog/production').mkdir(parents=True)
            original=(release.ROOT,release.OLD,release.OUT,release.PREP)
            try:
                release.ROOT=root
                release.OLD=root/'catalog/releases/7'
                release.OUT=root/'catalog/production'
                release.PREP=root/'catalog/preparation/m3d2a'
                evidence=release.PREP/'source-records.json'
                evidence.write_text(evidence.read_text()+' ')
                with self.assertRaisesRegex(AssertionError,'Pinned candidate evidence changed'):
                    release.build()
                evidence.write_bytes((original[3]/'source-records.json').read_bytes())
                candidates=release.PREP/'candidates.json'
                rows=json.loads(candidates.read_text())
                next(r for r in rows if r['candidateId']=='m3d2a-usda-167714')['proposedTurkishName']='Bal'
                candidates.write_text(json.dumps(rows,ensure_ascii=False))
                with self.assertRaisesRegex(AssertionError,'Catalog name collision'):
                    release.build()
                self.assertEqual(list(release.OUT.iterdir()),[])
            finally:
                release.ROOT,release.OLD,release.OUT,release.PREP=original


if __name__=='__main__':
    unittest.main()
