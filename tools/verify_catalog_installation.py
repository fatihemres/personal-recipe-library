#!/usr/bin/env python3
"""Exercise the real CLI against new temporary profiles; never opens user data.

Build catalog_import first. On macOS --deny-network applies a process-local
network denial (no changes to system connectivity or security settings).
"""
import argparse
import json
import pathlib
import subprocess
import tempfile
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--deny-network', action='store_true')
    parser.add_argument('--include-v4', action='store_true', help='Also verify the archived version-4 upgrade')
    parser.add_argument('--include-v5', action='store_true', help='Verify current version-6 release including archived version-5 upgrade')
    parser.add_argument('--include-v6', action='store_true', help='Verify release 7 including archived release-6 upgrade')
    parser.add_argument('--include-v7', action='store_true', help='Verify release 8 including archived release-7 upgrade')
    parser.add_argument('--include-v8', action='store_true', help='Verify release 9 including archived release-8 upgrade')
    parser.add_argument('--output', type=pathlib.Path, required=True)
    args = parser.parse_args()
    executable = ROOT / 'src-tauri/target/debug/examples/catalog_import'
    prefix = ['/usr/bin/sandbox-exec', '-p', '(version 1)(allow default)(deny network*)'] if args.deny_network else []
    results = []
    # Freshly allocated databases only. TemporaryDirectory owns their lifecycle.
    with tempfile.TemporaryDirectory(prefix='recipeatlas-catalog-') as base:
        previous_versions = (None, 2, 3, 4, 5, 6, 7, 8) if args.include_v8 else ((None, 2, 3, 4, 5, 6, 7) if args.include_v7 else ((None, 2, 3, 4, 5, 6) if args.include_v6 else ((None, 2, 3, 4, 5) if args.include_v5 else ((None, 2, 3, 4) if args.include_v4 else (None, 2, 3)))))
        for previous in previous_versions:
            profile = pathlib.Path(base) / str(previous or 'clean')
            manifests = []
            if previous:
                manifests.append(ROOT / f'catalog/releases/{previous}/manifest.json')
            target = ROOT / ('catalog/production/manifest.json' if (args.include_v7 or args.include_v8) else ('catalog/releases/7/manifest.json' if args.include_v6 else ('catalog/releases/6/manifest.json' if args.include_v5 else ('catalog/releases/5/manifest.json' if args.include_v4 else 'catalog/releases/4/manifest.json'))))
            manifests += [target] * 2
            for index, manifest in enumerate(manifests):
                start = time.perf_counter()
                process = subprocess.run(prefix + [str(executable), str(profile), str(manifest)], check=True, capture_output=True, text=True)
                elapsed = time.perf_counter() - start
                report = json.loads(process.stdout)
                repeat = index == len(manifests) - 1
                if repeat:
                    assert report['unchanged'] and report['inserted'] == 0 and report['updated'] == 0
                results.append(dict(previousVersion=previous, manifest=str(manifest.relative_to(ROOT)), repeat=repeat, elapsedMilliseconds=round(elapsed * 1000, 3), report=report))
    args.output.write_text(json.dumps(dict(networkDenied=args.deny_network, measurements='Local debug CLI wall time including process startup; not a universal performance guarantee.', runs=results), ensure_ascii=False, indent=2) + '\n')
    print(f'Verified {len(results)} real imports/opens in isolated profiles; network denied: {args.deny_network}')


if __name__ == '__main__':
    main()
