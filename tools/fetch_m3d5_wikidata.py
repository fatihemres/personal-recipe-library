#!/usr/bin/env python3
"""Pin selected public Wikidata entity revisions for M3D-5 research."""
import argparse
import hashlib
import json
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'catalog/preparation/m3d5'
SNAPSHOTS = OUT / 'wikidata-snapshots'
IDS = (
    'Q104218426', 'Q104246054', 'Q1721046', 'Q238306', 'Q690098',
    'Q822331', 'Q96098098', 'Q60855', 'Q519365', 'Q56342268',
    'Q654895', 'Q26877308', 'Q26877423', 'Q1448819', 'Q134967545',
    'Q12424061', 'Q5431859', 'Q1078150', 'Q12324500', 'Q900450',
    'Q902582', 'Q140136320', 'Q718680', 'Q139500622', 'Q138542182',
    'Q27670710', 'Q134967538', 'Q5140965', 'Q67890691', 'Q2916133',
    'Q3485291', 'Q3539556', 'Q548940', 'Q744729', 'Q5139704',
    'Q67778134', 'Q104704742', 'Q96240608', 'Q140606571', 'Q28208975',
    'Q192892', 'Q392001', 'Q1139302', 'Q633401', 'Q1377087', 'Q693187',
)
USER_AGENT = 'TarifAtlasiCatalogResearch/0.1 (https://github.com/fatihemres/personal-recipe-library)'


def fetch(url):
    request = urllib.request.Request(url, headers={'User-Agent': USER_AGENT})
    with urllib.request.urlopen(request, timeout=20) as response:
        return response.read()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fetch', action='store_true')
    args = parser.parse_args()
    index_path = OUT / 'wikidata-index.json'
    if args.fetch:
        SNAPSHOTS.mkdir(parents=True, exist_ok=True)
        previous = {entry['qid']: entry for entry in json.loads(index_path.read_text(encoding='utf-8'))} if index_path.exists() else {}
        index = []
        for qid in IDS:
            if qid in previous:
                index.append(previous[qid])
                continue
            current = json.loads(fetch(f'https://www.wikidata.org/wiki/Special:EntityData/{qid}.json'))['entities'][qid]
            revision = current['lastrevid']
            url = f'https://www.wikidata.org/wiki/Special:EntityData/{qid}.json?revision={revision}'
            raw = fetch(url)
            entity = json.loads(raw)['entities'][qid]
            assert entity['id'] == qid and entity['lastrevid'] == revision
            (SNAPSHOTS / f'{qid}.json').write_bytes(raw)
            index.append({'qid': qid, 'revision': revision, 'url': url,
                          'sha256': hashlib.sha256(raw).hexdigest()})
            time.sleep(0.1)
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
