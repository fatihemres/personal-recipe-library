#!/usr/bin/env python3
"""Release-7 reproducibility and boundary tests in disposable repository copies."""
import contextlib
import io
import json
from pathlib import Path
import shutil
import tempfile
import unittest
import prepare_m3d1_catalog as prepare
import report_m3d1_quality as quality


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.original = prepare.ROOT
        root = Path(self.temp.name)
        shutil.copytree(self.original/'catalog/releases/7', root/'catalog/production')
        shutil.copytree(self.original/'catalog/releases/7', root/'catalog/releases/7')
        shutil.copytree(self.original/'catalog/releases/6', root/'catalog/releases/6')
        prepare.ROOT = quality.ROOT = root
        prepare.OUT = root/'catalog/production'
        prepare.OLD = root/'catalog/releases/6'
        prepare.BATCH = prepare.OUT/'batches/m3d1-first-curated.tsv'
        self.addCleanup(self.restore)

    def restore(self):
        prepare.ROOT = quality.ROOT = self.original
        prepare.OUT = self.original/'catalog/production'
        prepare.OLD = self.original/'catalog/releases/6'
        prepare.BATCH = prepare.OUT/'batches/m3d1-first-curated.tsv'

    def test_offline_evidence_regeneration_is_byte_identical(self):
        names = ['ingredients.json', 'manifest.json', 'coverage.json', 'm3d1-usda-records.json', 'm3d1-curation-records.json']
        before = {name: (prepare.OUT/name).read_bytes() for name in names}
        with contextlib.redirect_stdout(io.StringIO()):
            prepare.main('--evidence')
        self.assertEqual(before, {name: (prepare.OUT/name).read_bytes() for name in names})
        quality.build()

    def test_tampered_source_and_candidate_fail(self):
        source = prepare.OUT/'m3d1-usda-records.json'
        source.write_text(source.read_text()+' ')
        with self.assertRaisesRegex(AssertionError, 'Evidence checksum'):
            prepare.main('--evidence')

    def test_changed_historical_identity_is_not_accepted(self):
        path = prepare.ROOT / 'catalog/releases/7/ingredients.json'
        data = json.loads(path.read_text())
        next(i for i in data['ingredients'] if i['key']=='honey')['names']['tr']='Wrong'
        path.write_text(json.dumps(data))
        with self.assertRaisesRegex(AssertionError, 'Historical row'):
            quality.build()

    def test_unresolved_decisions_and_deferred_identity_are_explicit(self):
        generated = quality.build()
        review = json.loads(generated[prepare.OUT/'m3d1-review.json'])
        self.assertEqual(len(review), 4)
        self.assertTrue(all(r['humanDecision'] is None for r in review))
        self.assertEqual(sum(not r['includedInRelease'] for r in review), 1)
        historic = json.loads((prepare.OUT/'localization-review.json').read_text())
        self.assertEqual(sum(r['decision']=='retain-pending-human-review' for r in historic), 3)


if __name__ == '__main__':
    unittest.main()
