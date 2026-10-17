#!/usr/bin/env python3
"""Measure release 9 with the real importer and disposable SQLite profiles."""
import argparse
import hashlib
import json
import sqlite3
import subprocess
import tempfile
import time
from pathlib import Path

from report_m3d1_coverage import ROOT, CLI, measure


def snapshot(path):
    db = sqlite3.connect(path)
    result = {
        'catalogIngredients': db.execute('SELECT count(*) FROM catalog_ingredients').fetchone()[0],
        'aliases': db.execute('SELECT count(*) FROM catalog_aliases').fetchone()[0],
        'categories': db.execute('SELECT count(*) FROM ingredient_categories').fetchone()[0],
        'personalIngredients': db.execute("SELECT count(*) FROM ingredients WHERE origin='personal'").fetchone()[0],
        'recipes': db.execute('SELECT count(*) FROM recipes').fetchone()[0],
        'drafts': db.execute('SELECT count(*) FROM editor_drafts').fetchone()[0],
        'integrity': db.execute('PRAGMA integrity_check').fetchone()[0],
        'foreignKeyViolations': len(db.execute('PRAGMA foreign_key_check').fetchall()),
    }
    db.close()
    return result


def personal_row_digests(path):
    db = sqlite3.connect(path)
    tables = ('recipes', 'recipe_ingredients', 'recipe_steps', 'editor_drafts',
              'draft_ingredients', 'preferences', 'personal_ingredient_favorites',
              'catalog_ingredient_favorites', 'ingredient_customizations',
              'ingredient_personal_aliases', 'personal_category_memberships')
    digests = {table: hashlib.sha256(repr(db.execute(f'SELECT * FROM {table} ORDER BY rowid').fetchall()).encode()).hexdigest()
               for table in tables}
    digests['personal_ingredients'] = hashlib.sha256(repr(db.execute("SELECT * FROM ingredients WHERE origin='personal' ORDER BY id").fetchall()).encode()).hexdigest()
    db.close()
    return digests


def import_seed(directory, manifest):
    start = time.perf_counter()
    process = subprocess.run([str(CLI), str(directory), str(manifest)], check=True, capture_output=True, text=True)
    return json.loads(process.stdout), round((time.perf_counter() - start) * 1000, 3)


def upgrade(previous):
    with tempfile.TemporaryDirectory(prefix='recipeatlas-m3d3-upgrade-') as name:
        directory = Path(name)
        before, _ = import_seed(directory, ROOT/f'catalog/releases/{previous}/manifest.json')
        original = snapshot(directory/'library.sqlite3')
        changed, elapsed = import_seed(directory, ROOT/'catalog/production/manifest.json')
        final = snapshot(directory/'library.sqlite3')
        repeat, _ = import_seed(directory, ROOT/'catalog/production/manifest.json')
        assert final['catalogIngredients'] == 745 and final['aliases'] == 269 and final['categories'] == 50
        assert final['integrity'] == 'ok' and final['foreignKeyViolations'] == 0
        assert repeat['unchanged'] and repeat['inserted'] == repeat['updated'] == 0
        assert changed['inserted'] == 745 - original['catalogIngredients']
        return dict(fromRelease=previous, initial=before, before=original, upgrade=changed, after=final, repeat=repeat, upgradeMilliseconds=elapsed)


def historical(source):
    # Operate only on a disposable SQLite backup of an already isolated profile.
    with tempfile.TemporaryDirectory(prefix='recipeatlas-m3d3-historical-') as name:
        directory = Path(name)
        origin = sqlite3.connect(f'file:{source}?mode=ro', uri=True)
        copy = sqlite3.connect(directory/'library.sqlite3')
        origin.backup(copy)
        copy.close()
        origin.close()
        before = snapshot(directory/'library.sqlite3')
        assert before['catalogIngredients'] == 648, 'Not the expected retained-preview profile'
        user_before = personal_row_digests(directory/'library.sqlite3')
        result, elapsed = import_seed(directory, ROOT/'catalog/production/manifest.json')
        after = snapshot(directory/'library.sqlite3')
        assert personal_row_digests(directory/'library.sqlite3') == user_before, 'Historical personal rows changed'
        repeat, _ = import_seed(directory, ROOT/'catalog/production/manifest.json')
        assert after['catalogIngredients'] == 746 and after['aliases'] == 269 and after['categories'] == 50
        assert after['integrity'] == 'ok' and after['foreignKeyViolations'] == 0
        assert repeat['unchanged']
        for key in ('personalIngredients', 'recipes', 'drafts'):
            assert before[key] == after[key], f'Historical user data changed: {key}'
        return dict(profile='isolated prior investigation copy; no personal values emitted', before=before, upgrade=result, after=after, repeat=repeat, personalRowsUnchanged=True, upgradeMilliseconds=elapsed)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--historical-profile', type=Path, help='Existing isolated 648-record profile copy, never the normal app profile')
    args = parser.parse_args()
    assert CLI.exists(), 'Build cargo example catalog_import first'
    old = measure(ROOT/'catalog/releases/8/manifest.json')
    current = measure(ROOT/'catalog/production/manifest.json')
    assert old['totals'] == {'catalog_ingredients': 647, 'catalog_aliases': 269, 'ingredient_categories': 50}
    assert current['totals'] == {'catalog_ingredients': 745, 'catalog_aliases': 269, 'ingredient_categories': 50}
    assert current['ingredientTypes'] == {'food': 624, 'beverage': 121}
    v7, v8 = upgrade(7), upgrade(8)
    assert v7['upgrade']['inserted'] == 177 and v8['upgrade']['inserted'] == 98
    report = dict(method='Real Rust CLI; disposable SQLite profiles; wall-time includes process startup. Direct category memberships overlap.', baseline=old, current=current, upgrades=[v7, v8])
    if args.historical_profile:
        report['historicalPreviewProfile'] = historical(args.historical_profile)
    (ROOT/'catalog/production/m3d3-sqlite-report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
    before = {c['key']: c['directCount'] for c in old['categories']}
    rows = ['# Release 9 measured category coverage', '', 'These counts come from fresh, isolated SQLite imports through the real Rust importer. Direct category memberships overlap. The 98 additions are 88 food and 10 beverage identities; 745 total identities, 269 aliases and 50 categories.', '', '| Category | Release 8 direct | Release 9 direct | Change |', '| --- | ---: | ---: | ---: |']
    for category in current['categories']:
        prior, now = before[category['key']], category['directCount']
        rows.append(f"| `{category['key']}` | {prior} | {now} | {now-prior:+d} |")
    rows += ['', 'Release 7→9 and 8→9 import, idempotency, integrity and foreign-key results are in `catalog/production/m3d3-sqlite-report.json`. Historical-preview results are included only when an isolated 648-record profile copy is supplied.', '', 'Remaining gaps: regional Turkish cheeses/ferments/greens; cuisine-specific South/East/Southeast Asian and MENA condiments; botanical tea and coffee forms; cocktail bitters and syrups. The pinned USDA legacy archive offers limited generic coverage for these; FoodOn term clearance and alternative licensed sources require further review. Seven earlier terminology/identity questions and 29 deferred current candidates are not approved.', '']
    (ROOT/'docs/CATALOG_M3D3_COVERAGE.md').write_text('\n'.join(rows))
    print('SQLite release 9: fresh 745, v7→9 +177, v8→9 +98, integrity/FK clean')


if __name__ == '__main__':
    main()
