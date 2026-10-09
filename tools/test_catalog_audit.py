#!/usr/bin/env python3
"""Audit safety tests using copies of real version-2 and version-3 artifacts."""
import contextlib
import io
import json
import pathlib
import shutil
import tempfile
import unittest
import audit_catalog


class CatalogAuditTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.original_root, self.original_out = audit_catalog.ROOT, audit_catalog.OUT
        self.addCleanup(self.restore)
        audit_catalog.ROOT = pathlib.Path(self.tmp.name)
        audit_catalog.OUT = audit_catalog.ROOT / 'catalog/production'
        shutil.copytree(self.original_out, audit_catalog.OUT)
        shutil.copytree(self.original_root / 'catalog/releases/2', audit_catalog.ROOT / 'catalog/releases/2')
        shutil.copytree(self.original_root / 'catalog/releases/3', audit_catalog.ROOT / 'catalog/releases/3')
        shutil.copyfile(self.original_root / 'catalog/sources.json', audit_catalog.ROOT / 'catalog/sources.json')
        shutil.copyfile(self.original_root / 'catalog/reference-sources.json', audit_catalog.ROOT / 'catalog/reference-sources.json')
        (audit_catalog.ROOT / 'docs').mkdir()

    def restore(self):
        audit_catalog.ROOT, audit_catalog.OUT = self.original_root, self.original_out

    def run_audit(self):
        with contextlib.redirect_stdout(io.StringIO()):
            audit_catalog.audit()

    def test_real_catalog_audit_and_stable_history(self):
        self.run_audit()
        report = json.loads((audit_catalog.OUT / 'quality-audit.json').read_text())
        self.assertEqual(report['auditedIdentities'], 508)
        self.assertEqual(report['stableHistoricalIds'], 482)
        self.assertEqual(report['linguisticReviewCount'], 18)
        self.assertEqual(report['independentlyHumanReviewed'], 0)

    def test_malformed_labels_aliases_source_units_and_ids_fail(self):
        path = audit_catalog.OUT / 'ingredients.json'
        original = json.loads(path.read_text())
        for defect in ('identity', 'label', 'alias', 'source', 'unit'):
            with self.subTest(defect=defect):
                data = json.loads(json.dumps(original))
                item = next(i for i in data['ingredients'] if i['key'] == 'sugar-granulated')
                if defect == 'identity':
                    item['id'] = 'changed'
                elif defect == 'label':
                    item['names']['tr'] = 'S\u0327eker'
                elif defect == 'alias':
                    item['aliases'].append(dict(locale='tr', name='Bal'))
                elif defect == 'source':
                    item['provenance'] = []
                else:
                    powder = next(i for i in data['ingredients'] if i['key'] == 'sugar-powdered')
                    powder['preferredUnit'] = 'mL'
                path.write_text(json.dumps(data))
                import hashlib
                manifest_path = audit_catalog.OUT / 'manifest.json'
                manifest = json.loads(manifest_path.read_text())
                manifest['artifact']['sha256'] = hashlib.sha256(path.read_bytes()).hexdigest()
                manifest_path.write_text(json.dumps(manifest))
                with self.assertRaises((ValueError, AssertionError)):
                    self.run_audit()

    def test_new_graph_provenance_type_and_duplicate_gates(self):
        path = audit_catalog.OUT / 'ingredients.json'
        original = json.loads(path.read_text())
        for defect in ('cycle', 'parent', 'category', 'version', 'external', 'type', 'alias', 'duplicate'):
            with self.subTest(defect=defect):
                data = json.loads(json.dumps(original))
                item = next(i for i in data['ingredients'] if i['key'] == 'sugar-granulated')
                if defect == 'cycle': data['categories'][0]['parent'] = data['categories'][0]['key']
                elif defect == 'parent': data['categories'][0]['parent'] = 'missing'
                elif defect == 'category': item['categories'] = ['missing']
                elif defect == 'version': item['provenance'][0]['sourceVersion'] = 'unapproved'
                elif defect == 'external': item['provenance'][0]['externalId'] = 'made-up'
                elif defect == 'type': item['categories'] = ['spirits']
                elif defect == 'alias': item['aliases'] = [dict(locale='tr', name='Özel'), dict(locale='tr', name='ÖZEL')]
                else: data['ingredients'].append(item)
                path.write_text(json.dumps(data))
                import hashlib
                manifest_path = audit_catalog.OUT / 'manifest.json'
                manifest = json.loads(manifest_path.read_text())
                manifest['artifact']['sha256'] = hashlib.sha256(path.read_bytes()).hexdigest()
                manifest_path.write_text(json.dumps(manifest))
                with self.assertRaises((ValueError, AssertionError)):
                    self.run_audit()

    def test_artifact_tampering_fails_without_regenerated_checksum(self):
        path = audit_catalog.OUT / 'ingredients.json'
        path.write_text(path.read_text() + ' ')
        with self.assertRaises(AssertionError):
            self.run_audit()


if __name__ == '__main__':
    unittest.main()
