#!/usr/bin/env python3
"""Verify Release-11 counts using the real Rust importer and isolated SQLite."""
import argparse
import json
import subprocess
import tempfile
import time
from pathlib import Path

from report_m3d1_coverage import CLI, ROOT, measure

PREVIOUS = ROOT / 'catalog/releases/10/manifest.json'
CURRENT = ROOT / 'catalog/production/manifest.json'


def upgrade(version):
    with tempfile.TemporaryDirectory(prefix='recipeatlas-m3d5-upgrade-') as name:
        directory = Path(name)
        prior = ROOT / f'catalog/releases/{version}/manifest.json'
        first = json.loads(subprocess.run([str(CLI), str(directory), str(prior)], check=True,
                                          capture_output=True, text=True).stdout)
        start = time.perf_counter()
        updated = json.loads(subprocess.run([str(CLI), str(directory), str(CURRENT)], check=True,
                                            capture_output=True, text=True).stdout)
        elapsed = round((time.perf_counter() - start) * 1000, 3)
        repeat = json.loads(subprocess.run([str(CLI), str(directory), str(CURRENT)], check=True,
                                           capture_output=True, text=True).stdout)
        assert updated['version'] == 11 and updated['inserted'] == 854 - first['validated']
        assert repeat['unchanged'] and repeat['inserted'] == repeat['updated'] == 0
        return dict(fromRelease=version, before=first['validated'], inserted=updated['inserted'],
                    updated=updated['updated'], unchangedOnRepeat=True, upgradeMilliseconds=elapsed)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    baseline, current = measure(PREVIOUS), measure(CURRENT)
    coverage = json.loads((ROOT / 'catalog/production/coverage.json').read_text(encoding='utf-8'))
    assert baseline['totals'] == {'catalog_ingredients': 795, 'catalog_aliases': 269, 'ingredient_categories': 50}
    assert current['totals'] == {'catalog_ingredients': 854, 'catalog_aliases': 269, 'ingredient_categories': 50}
    assert baseline['ingredientTypes'] == {'food': 666, 'beverage': 129}
    assert current['ingredientTypes'] == {'food': 666, 'beverage': 188}
    assert current['integrity'] == 'ok' and current['foreignKeyViolations'] == 0
    assert coverage['total'] == 854 and coverage['aliases'] == 269 and coverage['categoryNodes'] == 50
    direct = {category['key']: category['directCount'] for category in current['categories']}
    assert {key: count for key, count in direct.items() if count} == coverage['categoryMembershipCounts']
    upgrades = [upgrade(version) for version in (7, 8, 9, 10)]
    old = {category['key']: category['directCount'] for category in baseline['categories']}
    lines = ['# M3D-5 beverage coverage and gap matrix', '',
             '**Release 11: 854 canonical identities, 666 food / 188 beverage, 269 aliases and 50 categories.**'
             ' Fifty-nine new beverage-oriented identities were accepted after screening 85 source candidates.'
             ' All 795 Release-10 identity objects and all category objects remain unchanged.', '',
             'Direct category memberships overlap and must not be summed into the 188 unique beverage identities.', '',
             '| Beverage category | Release 10 direct | Release 11 direct | Change |', '| --- | ---: | ---: | ---: |']
    for key in ('coffee', 'tea', 'infusions', 'syrups', 'mixers', 'bitters', 'liqueurs',
                'spirits', 'traditional-drinks', 'garnishes', 'juices', 'wines',
                'soft-drinks', 'other-beverages', 'water', 'beers'):
        lines.append(f'| `{key}` | {old[key]} | {direct[key]} | {direct[key]-old[key]:+d} |')
    lines += ['', 'The release adds bean and ground coffee forms; matcha, white, pu-erh, jasmine and genmaicha teas;'
              ' orange bitters, ginger beer, falernum, cordials, syrups and garnishes; several fruit juices and wine varieties;'
              ' and salep flour as a traditional beverage input. English source identities and original Turkish editorial labels are separate.', '',
              '**Remaining high-value gaps:** independently verified dry rooibos and herbal materials, more coffee extracts and concentrates,'
              ' vermouth identities, culturally specific şerbet bases, additional alcohol-free mixers, and non-branded liqueur types.'
              ' The 16 deferred candidates have explicit reasons in `catalog/preparation/m3d5/candidates.json`; none is in the runtime seed.', '',
              'This is a real SQLite-backed count. `catalog/production/m3d5-sqlite-report.json` records isolated fresh/upgrade'
              ' integrity checks and local debug-CLI durations, which include process startup and are not cross-device benchmarks.']
    markdown = '\n'.join(lines) + '\n'
    path = ROOT / 'docs/CATALOG_M3D5_COVERAGE.md'
    if args.check:
        assert path.read_text(encoding='utf-8') == markdown, f'Stale Release-11 coverage: {path}'
    else:
        path.write_text(markdown, encoding='utf-8')
        report = dict(method='Real Rust catalog_import CLI on disposable SQLite profiles; category counts are direct unique memberships.',
                      baseline=baseline, current=current, upgrades=upgrades)
        (ROOT / 'catalog/production/m3d5-sqlite-report.json').write_text(
            json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print('SQLite Release 11: 854 identities, 666 food / 188 beverage, 269 aliases, 50 categories; v7-v10 upgrades verified')


if __name__ == '__main__':
    main()
