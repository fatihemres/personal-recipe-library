#!/usr/bin/env python3
"""Measure release 7 and 8 through the real importer in isolated SQLite files."""
import json
import sqlite3
import subprocess
import tempfile
from pathlib import Path

from report_m3d1_coverage import measure, ROOT, CLI


def upgrade():
    with tempfile.TemporaryDirectory(prefix='recipeatlas-m3d2b-upgrade-') as name:
        path = Path(name)
        previous = json.loads(subprocess.run([str(CLI), str(path), str(ROOT/'catalog/releases/7/manifest.json')], check=True, text=True, capture_output=True).stdout)
        changed = json.loads(subprocess.run([str(CLI), str(path), str(ROOT/'catalog/releases/8/manifest.json')], check=True, text=True, capture_output=True).stdout)
        repeat = json.loads(subprocess.run([str(CLI), str(path), str(ROOT/'catalog/releases/8/manifest.json')], check=True, text=True, capture_output=True).stdout)
        db = sqlite3.connect(path/'library.sqlite3')
        counts = {table: db.execute(f'SELECT count(*) FROM {table}').fetchone()[0] for table in ('catalog_ingredients','catalog_aliases','ingredient_categories')}
        integrity = db.execute('PRAGMA integrity_check').fetchone()[0]
        violations = len(db.execute('PRAGMA foreign_key_check').fetchall())
        db.close()
    assert previous['inserted'] == 568 and changed['inserted'] == 79 and changed['updated'] == 60
    assert repeat['unchanged'] and repeat['inserted'] == repeat['updated'] == 0
    assert counts == {'catalog_ingredients':647,'catalog_aliases':269,'ingredient_categories':50}
    assert integrity == 'ok' and violations == 0
    return dict(fromRelease=7,toRelease=8,initialImport=previous,upgrade=changed,repeat=repeat,counts=counts,integrity=integrity,foreignKeyViolations=violations)


def main():
    old = measure(ROOT/'catalog/releases/7/manifest.json')
    current = measure(ROOT/'catalog/releases/8/manifest.json')
    upgraded = upgrade()
    assert old['totals'] == {'catalog_ingredients':568,'catalog_aliases':269,'ingredient_categories':50}
    assert current['totals'] == {'catalog_ingredients':647,'catalog_aliases':269,'ingredient_categories':50}
    assert current['ingredientTypes'] == {'food':536,'beverage':111}
    report = dict(method='Real Rust catalog_import CLI and disposable SQLite profiles; direct category membership counts overlap.',baseline=old,current=current,upgrade=upgraded)
    (ROOT/'catalog/releases/8/m3d2b-sqlite-report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    baseline = {c['key']:c['directCount'] for c in old['categories']}
    rows = ['# M3D-2B measured release 8 coverage','','All counts below come from fresh isolated SQLite imports through the real Rust catalog importer. Category memberships overlap and must not be summed as a catalog total.','','Release 7: **568 identities, 464 food / 104 beverage, 269 aliases, 50 categories.**','','Release 8: **647 identities, 536 food / 111 beverage, 269 aliases, 50 categories.** The 79 additions are 72 food and 7 beverage identities. No new aliases or category nodes were introduced.','','Release-7→8 upgrade inserted 79 and updated source-version provenance for 60 existing M3D-1 identities; names, IDs and categories did not change. Repeated import changed zero. SQLite integrity was `ok` with zero foreign-key violations. The normal user profile was not opened.','','| Category | Parent | Release 7 direct | Release 8 direct | Change |','| --- | --- | ---: | ---: | ---: |']
    for category in current['categories']:
        before=baseline[category['key']]
        after=category['directCount']
        rows.append(f"| `{category['key']}` | `{category['parent'] or '—'}` | {before} | {after} | {after-before:+d} |")
    rows += ['','These direct counts are database measurements, not human linguistic validation. The 27 deferred USDA concepts, 88 FoodOn concepts and seven historical questions remain outside this release. Regional Turkish, specialty bar/bitters and dairy-alternative gaps still need separate legally cleared sources.','']
    (ROOT/'docs/CATALOG_M3D2B_COVERAGE.md').write_text('\n'.join(rows))
    print(f"SQLite release 8: {current['totals']}; release 7→8 inserted {upgraded['upgrade']['inserted']}; integrity {upgraded['integrity']}")


if __name__=='__main__':
    main()
