#!/usr/bin/env python3
"""Measure Release-10 coverage in isolated real SQLite profiles."""
import argparse
import json
import subprocess
import tempfile
import time
from pathlib import Path

from report_m3d1_coverage import CLI, ROOT, measure


def upgrade(version):
    with tempfile.TemporaryDirectory(prefix='recipeatlas-m3d4-upgrade-') as name:
        directory = Path(name)
        prior = ROOT / f'catalog/releases/{version}/manifest.json'
        current = ROOT / 'catalog/releases/10/manifest.json'
        first = json.loads(subprocess.run([str(CLI), str(directory), str(prior)], check=True,
                                          capture_output=True, text=True).stdout)
        start = time.perf_counter()
        updated = json.loads(subprocess.run([str(CLI), str(directory), str(current)], check=True,
                                            capture_output=True, text=True).stdout)
        elapsed = round((time.perf_counter() - start) * 1000, 3)
        repeat = json.loads(subprocess.run([str(CLI), str(directory), str(current)], check=True,
                                           capture_output=True, text=True).stdout)
        assert first['version'] == version and updated['version'] == 10
        assert updated['inserted'] == 795 - first['validated']
        assert repeat['unchanged'] and repeat['inserted'] == repeat['updated'] == 0
        return dict(fromRelease=version, before=first['validated'], inserted=updated['inserted'],
                    updated=updated['updated'], unchangedOnRepeat=True, upgradeMilliseconds=elapsed)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='Verify live SQLite counts without writing measured outputs')
    args = parser.parse_args()
    baseline = measure(ROOT / 'catalog/releases/9/manifest.json')
    current = measure(ROOT / 'catalog/releases/10/manifest.json')
    expected = {'catalog_ingredients': 795, 'catalog_aliases': 269, 'ingredient_categories': 50}
    assert baseline['totals'] == {'catalog_ingredients': 745, 'catalog_aliases': 269, 'ingredient_categories': 50}
    assert current['totals'] == expected
    assert current['ingredientTypes'] == {'food': 666, 'beverage': 129}
    assert current['integrity'] == 'ok' and current['foreignKeyViolations'] == 0
    coverage = json.loads((ROOT / 'catalog/releases/10/coverage.json').read_text(encoding='utf-8'))
    assert coverage['total'] == 795 and coverage['aliases'] == 269 and coverage['categoryNodes'] == 50
    direct = {category['key']: category['directCount'] for category in current['categories']}
    assert {key: count for key, count in direct.items() if count} == coverage['categoryMembershipCounts']
    assert direct['turkish-regional'] == 27
    upgrades = [upgrade(version) for version in (7, 8, 9)]
    if not args.check:
        report = dict(method='Real Rust catalog_import CLI on disposable SQLite profiles; category counts are direct unique memberships and overlap.',
                      baseline=baseline, current=current, upgrades=upgrades)
        (ROOT / 'catalog/releases/10/m3d4-sqlite-report.json').write_text(
            json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        old = {category['key']: category['directCount'] for category in baseline['categories']}
        lines = ['# Release 10 SQLite-backed category coverage', '',
                 'Fresh isolated SQLite imports through the real Rust importer measured 745 Release-9 and 795 Release-10 identities. Direct category memberships overlap, so category counts cannot be summed into the catalog total.', '',
                 '**Release 10:** 795 identities (666 food / 129 beverage), 269 aliases, 50 categories. Fifty identities were added: 35 USDA Foundation and 15 pinned Wikidata concepts. No prior canonical object, alias, or category node changed.', '',
                 '| Category | Release 9 direct | Release 10 direct | Change |', '| --- | ---: | ---: | ---: |']
        for key, count in sorted(direct.items()):
            lines.append(f'| `{key}` | {old[key]} | {count} | {count-old[key]:+d} |')
        lines += ['', '**Turkish regional:** 16 → 27 direct memberships, including cheeses, cured meat, fermented tarhana, spice and mastic forms. These are editorially identified culinary ingredients, not claims of protected-origin certification.', '',
                  'The 50 additions do not achieve the aspirational 900–1,000 total. Remaining gaps include regional herbs and grains, North African and Latin American pantry staples, East/Southeast Asian sauces, botanical infusions and bar ingredients. FoodOn term lineage is still unresolved. The separate source and decision audit records each exclusion.', '',
                  'Fresh/import/upgrade integrity and foreign-key results are in `catalog/production/m3d4-sqlite-report.json`. Times there include debug CLI startup and are environment-specific.', '']
        (ROOT / 'docs/CATALOG_M3D4_COVERAGE.md').write_text('\n'.join(lines), encoding='utf-8')
    print('SQLite Release 10: 795 identities, 666 food / 129 beverage, 269 aliases, 50 categories; v7/v8/v9 upgrades verified')


if __name__ == '__main__':
    main()
