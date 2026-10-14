#!/usr/bin/env python3
"""Measure unchanged release 7 in an isolated SQLite profile; write preparation only."""
import json
import sqlite3
import subprocess
import tempfile
from pathlib import Path
from collections import Counter
from prepare_m3d2a_candidates import ROOT, OUT, check_baseline, encoded, read


def measure():
    check_baseline()
    cli = ROOT / 'src-tauri/target/debug/examples/catalog_import'
    with tempfile.TemporaryDirectory(prefix='recipeatlas-m3d2a-') as temp:
        result = subprocess.run([str(cli), temp, str(ROOT / 'catalog/production/manifest.json')], text=True, capture_output=True, check=True)
        installed = json.loads(result.stdout)
        db = sqlite3.connect(Path(temp) / 'library.sqlite3')
        totals = {table: db.execute(f'SELECT count(*) FROM {table}').fetchone()[0] for table in ['catalog_ingredients','catalog_aliases','ingredient_categories']}
        assert list(totals.values()) == [568, 269, 50]
        types = dict(db.execute('SELECT ingredient_type,count(*) FROM catalog_ingredients GROUP BY ingredient_type'))
        assert types == {'food':464,'beverage':104}
        candidates = read('candidates.json')
        counts = Counter(c for r in candidates for c in r['proposedCategories'])
        higher = Counter(c for r in candidates if r['proposedImportStatus']=='proposed-after-editorial-review' for c in r['proposedCategories'])
        categories = []
        for cid,key,tr,en,parent in db.execute('SELECT c.id,c.stable_key,c.name_tr,c.name_en,p.stable_key FROM ingredient_categories c LEFT JOIN ingredient_categories p ON p.id=c.parent_id ORDER BY c.stable_key'):
            direct = db.execute('SELECT count(DISTINCT ingredient_id) FROM catalog_category_memberships WHERE category_id=?',(cid,)).fetchone()[0]
            descendants = db.execute('''WITH RECURSIVE tree(id) AS (SELECT ? UNION SELECT c.id FROM ingredient_categories c JOIN tree t ON c.parent_id=t.id)
             SELECT count(DISTINCT m.ingredient_id) FROM catalog_category_memberships m JOIN tree t ON t.id=m.category_id''',(cid,)).fetchone()[0]
            categories.append(dict(key=key,tr=tr,en=en,parent=parent,direct=direct,includingDescendants=descendants,candidates=counts[key],higherConfidenceCandidates=higher[key]))
        assert db.execute('PRAGMA integrity_check').fetchone()[0]=='ok'
        assert not db.execute('PRAGMA foreign_key_check').fetchall()
        db.close()
    check_baseline()
    return dict(release=installed['version'],totals=totals,types=types,categories=categories,integrity='ok',foreignKeyViolations=0,method='Fresh temporary SQLite profile; real Rust importer; unique direct and descendant membership counts overlap.')


if __name__=='__main__':
    report=measure()
    (OUT/'coverage.json').write_text(encoded(report))
    lines=['# M3D-2A measured category coverage','',report['method'],'','Release 7 remains **568 identities: 464 food / 104 beverage; 269 aliases; 50 categories**. Candidate counts are proposals, not installed additions. Higher-confidence is editorial screening, not human approval.','', 'The food/beverage root membership totals differ from ingredient-type totals because an identity may belong to both category trees; neither is an error or an additional identity.', '', '| Category | Parent | Direct | With descendants | Candidates | Higher-confidence proposals |','|---|---|---:|---:|---:|---:|']
    for r in report['categories']:
        lines.append(f"| {r['key']} | {r['parent'] or '—'} | {r['direct']} | {r['includingDescendants']} | {r['candidates']} | {r['higherConfidenceCandidates']} |")
    lines+=['','See CATALOG_M3D2A_PREPARATION.md for cuisine gaps, source restrictions and acceptance gates. No catalog version 8 exists in this preparation.','']
    (ROOT/'docs/CATALOG_M3D2A_COVERAGE.md').write_text('\n'.join(lines))
    print(json.dumps({'totals':report['totals'],'types':report['types'],'integrity':report['integrity']}))
