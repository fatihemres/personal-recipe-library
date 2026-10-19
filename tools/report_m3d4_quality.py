#!/usr/bin/env python3
"""Generate Release-10 source, decision and structural-quality report."""
import argparse
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'catalog/production'


def load(path):
    return json.loads(path.read_text(encoding='utf-8'))


def build():
    old = {item['key']: item for item in load(ROOT / 'catalog/releases/9/ingredients.json')['ingredients']}
    current = {item['key']: item for item in load(OUT / 'ingredients.json')['ingredients']}
    candidates = load(OUT / 'm3d4-decisions.json')
    manifest = load(OUT / 'manifest.json')
    audit = load(OUT / 'quality-audit.json')
    coverage = load(OUT / 'coverage.json')
    assert len(old) == 745 and len(current) == 795 and manifest['version'] == 10
    assert all(current[key] == item for key, item in old.items())
    assert not audit['structuralErrors'] and not audit['aliasCollisionCandidates']
    assert Counter(item['decision'] for item in candidates) == {'ACCEPT': 50, 'DEFER': 9, 'EXISTING': 4, 'REJECT': 2}
    added = [item for key, item in sorted(current.items()) if key not in old]
    lines = ['# M3D-4 Release 10 quality and decision audit', '',
             '**50 new canonical identities; 795 total (666 food / 129 beverage), 269 aliases, 50 categories.** All 745 Release-9 ingredient objects and every category object are byte-equivalent as parsed data. No previous source/version evidence is rewritten.', '',
             'The 65 newly screened candidates resolve to 50 ACCEPT, 9 DEFER, 4 EXISTING and 2 REJECT. The 29 earlier Release-9 deferrals and seven historical terminology/identity questions remain separate and unresolved. Automated editorial decisions are not independent human linguistic approval.', '',
             'The structural audit reports zero errors, zero exact alias collisions and zero new capitalization/translation warnings. Its 37 same-English-base concept groups are review hints, not proven duplicates. New additions have no aliases, nutrition, density, allergen or ABV observations.', '',
             '## Source and identity boundaries', '',
             '- USDA Foundation Foods April 2026: 35 selected CC0 descriptors with official FDC IDs, original ZIP SHA-256 `186e988ec542e913f51ef62b86a47758e8cdd0d1dc3889e7b055581f3c09c77a`. This is a different upstream data type and immutable source/version from prior SR Legacy selections.',
             '- Wikidata: 15 individually screened CC0 structured QID revisions. The pinned 24-entity candidate evidence stores exact revision URLs and original response checksums; no external photographs or linked copyrighted text are included.',
             '- Tarif Atlası: bilingual labels, category choices and unit defaults are separately authored curation, not claimed as USDA translations or independently approved terminology.',
             '- FoodOn, Open Food Facts, TürKomp and TheCocktailDB contributed zero bundled records. TÜRKPATENT and Turkish ministry pages were identity references, not redistributed datasets; protected-origin status is not inferred for a user-selected generic ingredient.', '',
             '## Accepted identities', '',
             '| Stable key | Turkish editorial label | English editorial label | Categories | Primary source |',
             '| --- | --- | --- | --- | --- |']
    for item in added:
        source = next(p for p in item['provenance'] if p['sourceId'] != 'recipeatlas-curation-m3d4')
        url = (f"https://fdc.nal.usda.gov/food-details/{source['externalId']}/nutrients"
               if source['sourceId'] == 'usda-foundation-m3d4'
               else next(c['sourceUrl'] for c in candidates if c['sourceIdentifier'] == source['externalId']))
        lines.append(f"| `{item['key']}` | {item['names']['tr']} | {item['names']['en']} | {', '.join(item['categories'])} | [{source['externalId']}]({url}) |")
    lines += ['', '## Excluded or pending candidates', '',
              '| Source ID | Decision | Rationale |', '| --- | --- | --- |']
    for candidate in candidates:
        if candidate['decision'] != 'ACCEPT':
            lines.append(f"| [{candidate['sourceIdentifier']}]({candidate['sourceUrl']}) | {candidate['decision']} | {candidate['reason']} |")
    lines += ['', 'Potentially close forms such as canned versus dry beans, concentrated versus fresh juice, coarse versus fine semolina, and regional versus generic dairy remain separate only where source identity and culinary use justify it. Remaining source rights and terminology issues are detailed in `docs/CATALOG_M3D4_SOURCE_EVALUATION.md`.', '']
    report = dict(release=10, baselineManifestSha256=coverage['inheritedRelease9ManifestSha256'],
                  additions=[dict(key=i['key'], id=i['id'], sourceId=next(p['sourceId'] for p in i['provenance'] if p['sourceId'] != 'recipeatlas-curation-m3d4'),
                                  externalId=next(p['externalId'] for p in i['provenance'] if p['sourceId'] != 'recipeatlas-curation-m3d4')) for i in added],
                  decisionCounts=dict(Counter(c['decision'] for c in candidates)),
                  sourceVersions=[dict(id=s['id'], version=s['version'], artifactSha256=s['artifactSha256']) for s in manifest['sources']],
                  structuralErrors=audit['structuralErrors'], exactAliasCollisions=audit['aliasCollisionCandidates'],
                  possibleConceptGroups=len(audit['conceptReviewCandidates']), olderDeferredCandidates=29,
                  historicalUnresolvedReviews=7, independentlyHumanApprovedAdditions=0)
    return {ROOT / 'docs/CATALOG_M3D4_QUALITY_AUDIT.md': '\n'.join(lines),
            OUT / 'm3d4-provenance-report.json': json.dumps(report, ensure_ascii=False, indent=2) + '\n'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    for path, content in build().items():
        if args.check:
            assert path.read_text(encoding='utf-8') == content, f'Stale M3D-4 audit artifact: {path}'
        else:
            path.write_text(content, encoding='utf-8')
    print('Release 10 quality: 50 accepted, 9 deferred, 4 existing, 2 rejected; structural audit clear')


if __name__ == '__main__':
    main()
