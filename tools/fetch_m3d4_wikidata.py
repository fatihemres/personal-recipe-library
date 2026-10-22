#!/usr/bin/env python3
"""Pin selected Wikidata entity revisions for M3D-4; no runtime network use."""
import argparse
import hashlib
import json
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'catalog/preparation/m3d4'
SNAPSHOTS = OUT / 'wikidata-snapshots'
IDS = (
    'Q3183584', 'Q6062076', 'Q6027754', 'Q478654', 'Q2118339',
    'Q3668413', 'Q1866347', 'Q12812990', 'Q148306', 'Q3337007',
    'Q27943', 'Q2056399', 'Q1432724', 'Q966327', 'Q699637',
    'Q1196289', 'Q67207722', 'Q734727', 'Q465554', 'Q318367',
    'Q16620197', 'Q235169', 'Q108946643', 'Q1817099',
)
USER_AGENT = 'TarifAtlasiCatalogResearch/0.1 (https://github.com/fatihemres/personal-recipe-library)'


def fetch(url):
    request = urllib.request.Request(url, headers={'User-Agent': USER_AGENT})
    with urllib.request.urlopen(request, timeout=15) as response:
        return response.read()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fetch', action='store_true', help='Retrieve public entities and pin exact revisions')
    args = parser.parse_args()
    index_path = OUT / 'wikidata-index.json'
    if args.fetch:
        SNAPSHOTS.mkdir(parents=True, exist_ok=True)
        index = []
        for qid in IDS:
            current = json.loads(fetch(f'https://www.wikidata.org/wiki/Special:EntityData/{qid}.json'))['entities'][qid]
            revision = current['lastrevid']
            url = f'https://www.wikidata.org/wiki/Special:EntityData/{qid}.json?revision={revision}'
            raw = fetch(url)
            entity = json.loads(raw)['entities'][qid]
            assert entity['id'] == qid and entity['lastrevid'] == revision
            (SNAPSHOTS / f'{qid}.json').write_bytes(raw)
            index.append({'qid': qid, 'revision': revision, 'url': url,
                          'sha256': hashlib.sha256(raw).hexdigest()})
            time.sleep(0.15)
        index_path.write_text(json.dumps(index, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    index = json.loads(index_path.read_text(encoding='utf-8'))
    assert [entry['qid'] for entry in index] == list(IDS)
    for entry in index:
        raw = (SNAPSHOTS / f"{entry['qid']}.json").read_bytes()
        assert hashlib.sha256(raw).hexdigest() == entry['sha256']
        entity = json.loads(raw)['entities'][entry['qid']]
        assert entity['id'] == entry['qid'] and entity['lastrevid'] == entry['revision']
        assert entry['url'].endswith(f"?revision={entry['revision']}")
    print(f'{len(index)} pinned Wikidata entity revisions verified')


if __name__ == '__main__':
    main()
