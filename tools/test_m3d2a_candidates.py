#!/usr/bin/env python3
"""Offline preparation gates; no personal databases or production writes."""
import copy
import csv
import io
import json
import unittest
import zipfile
import xml.etree.ElementTree as ET
from pathlib import Path
from prepare_m3d2a_candidates import ROOT, OUT, read, build, collisions, normalize, check_baseline


class CandidateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog = json.loads((ROOT/'catalog/releases/7/ingredients.json').read_text())
        cls.rows = read('candidates.json')
        cls.evidence = read('source-records.json')
        cls.selection = list(csv.DictReader(io.StringIO((OUT/'selection.tsv').read_text()), delimiter='\t'))
        cls.sources = {s['id']:s for s in read('sources.json')}

    def test_production_and_history_unchanged(self):
        check_baseline()
        self.assertEqual(len(self.catalog['ingredients']),568)
        self.assertEqual(sum(len(r['aliases']) for r in self.catalog['ingredients']),269)
        self.assertEqual(len(self.catalog['categories']),50)

    def test_traceable_unique_inventory(self):
        self.assertEqual(len(self.rows),194)
        self.assertEqual(len({r['candidateId'] for r in self.rows}),194)
        evidence = {(e['source'],e['id']):e for e in self.evidence}
        for r in self.rows:
            e=evidence[r['source'],r['sourceIdentifier']]
            self.assertEqual(r['sourceEnglishLabel'],e['label'])
            self.assertEqual(r['sourceUrl'],e['url'])
            self.assertTrue(r['proposedTurkishName'])
            self.assertEqual(r['observations'],[])
            self.assertIsNone(r['humanDecision'])
            self.assertFalse(r['importAuthorized'])

    def test_unreviewed_sources_cannot_be_high_confidence_import_proposals(self):
        self.assertEqual(sum(r['source']=='foodon' for r in self.rows),88)
        for r in self.rows:
            if r['source']=='foodon':
                self.assertNotEqual(r['proposedImportStatus'],'proposed-after-editorial-review')
        fake = copy.deepcopy(self.selection[:1]); fake[0]['source']='unapproved'
        with self.assertRaises(KeyError):
            build(fake,self.evidence[:1],self.catalog,self.sources)

    def test_invalid_category_and_duplicate_candidate_fail(self):
        bad=copy.deepcopy(self.selection); bad[0]['categories']='not-a-category'
        with self.assertRaises(AssertionError): build(bad,self.evidence,self.catalog,self.sources)
        with self.assertRaises(AssertionError): build(self.selection+[self.selection[0]],self.evidence+[self.evidence[0]],self.catalog,self.sources)

    def test_existing_usda_identity_detected_via_provenance(self):
        original=next(i for i in self.catalog['ingredients'] if any(p['sourceId'].startswith('usda') for p in i['provenance']))
        p=next(p for p in original['provenance'] if p['sourceId'].startswith('usda'))
        result=collisions({'source':'usda','id':p['externalId'],'tr':'Unrelated test name'},{'label':'Unrelated label'},self.catalog['ingredients'])
        self.assertTrue(any(r['kind']=='existing-source-identity' and r['id']==original['id'] for r in result))

    def test_turkish_case_and_unicode_alias_collision(self):
        self.assertEqual(normalize('ŞEKER'),normalize('şeker'))
        self.assertEqual(normalize('İNCİR'),normalize('incir'))
        self.assertEqual(normalize('IŞIK'),normalize('ışık'))
        self.assertNotEqual(normalize('I'),normalize('i'))
        self.assertEqual(normalize('S\u0327eker'),normalize('Şeker'))
        item=copy.deepcopy(self.catalog['ingredients'][0]); item['aliases']=[{'locale':'tr','name':'Şeker'}]
        result=collisions({'source':'foodon','id':'test','tr':'ŞEKER'},{'label':'Example'},[item])
        self.assertEqual(result[0]['kind'],'exact-name-or-alias')

    def test_legacy_reviews_preserved(self):
        reviews=read('prior-review.json')
        self.assertEqual({r['key'] for r in reviews},{'usda-sr-171319','usda-sr-170874','usda-sr-168412','usda-sr-168472','usda-sr-169406','usda-sr-170007','usda-sr-174208'})
        self.assertTrue(all(r['humanDecision'] is None for r in reviews))
        self.assertFalse(next(r for r in reviews if r['key']=='usda-sr-169406')['includedInRelease'])
        self.assertFalse(any(r['sourceIdentifier']=='169406' for r in self.rows))

    def test_counts_and_editorial_collision_decisions(self):
        report=read('quality-report.json')
        self.assertEqual(sum(report['byStatus'].values()),194)
        self.assertEqual(report['higherConfidenceByType'],{'food':61,'beverage':5})
        review=read('collision-review.json')
        self.assertEqual(sum(v['classification']=='probable-existing-concept' for v in review.values()),3)
        self.assertEqual(sum(v['classification']=='reject-brand-specific' for v in review.values()),1)
        self.assertTrue(all(not r['importAuthorized'] for r in self.rows))
        self.assertEqual(len(read('candidate-pairs.json')),1)

    def test_coverage_counts_are_not_sum_of_memberships(self):
        coverage=read('coverage.json')
        self.assertEqual(coverage['totals']['catalog_ingredients'],568)
        self.assertEqual(coverage['integrity'],'ok')
        self.assertEqual(coverage['foreignKeyViolations'],0)
        for c in coverage['categories']:
            self.assertGreaterEqual(c['includingDescendants'],c['direct'])
        self.assertGreater(sum(c['direct'] for c in coverage['categories']),568)

    def test_exported_workbook_review_controls_and_rows(self):
        ns = {'s':'http://schemas.openxmlformats.org/spreadsheetml/2006/main'}
        with zipfile.ZipFile(ROOT/'docs/INGREDIENT_M3D2A_CANDIDATE_REVIEW.xlsx') as z:
            workbook = ET.fromstring(z.read('xl/workbook.xml'))
            self.assertEqual(len(workbook.findall('s:sheets/s:sheet',ns)),4)
            strings = []
            if 'xl/sharedStrings.xml' in z.namelist():
                strings = [''.join(t.text or '' for t in si.iter('{'+ns['s']+'}t')) for si in ET.fromstring(z.read('xl/sharedStrings.xml'))]
            def value(cell):
                v=cell.find('s:v',ns)
                if cell.get('t')=='s': return strings[int(v.text)]
                if cell.get('t')=='inlineStr': return ''.join(t.text or '' for t in cell.findall('.//s:t',ns))
                return v.text if v is not None else ''
            identifiers=[]
            for index, expected in [(1,66),(2,135),(3,50),(4,7)]:
                sheet=ET.fromstring(z.read(f'xl/worksheets/sheet{index}.xml'))
                pane=sheet.find('s:sheetViews/s:sheetView/s:pane',ns)
                self.assertEqual(pane.get('ySplit'),'5')
                self.assertEqual(pane.get('state'),'frozen')
                rows=[r for r in sheet.findall('s:sheetData/s:row',ns) if int(r.get('r'))>=6]
                self.assertEqual(len(rows),expected)
                if index<=2:
                    for row in rows:
                        for cell in row:
                            if cell.get('r').startswith(('E','F')): self.assertEqual(value(cell),'')
                            if cell.get('r').startswith('A'): identifiers.append(value(cell))
                table=ET.fromstring(z.read(f'xl/tables/table{index}.xml'))
                self.assertIsNotNone(table.find('s:autoFilter',ns))
            self.assertEqual(set(identifiers),{r['candidateId'] for r in self.rows}|{r['key'] for r in read('prior-review.json')})


if __name__=='__main__':
    unittest.main()
