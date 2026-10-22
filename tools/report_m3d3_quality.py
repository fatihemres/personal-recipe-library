#!/usr/bin/env python3
"""Reproduce Release-9 human-readable decision and provenance audit."""
import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT/'catalog/releases/9'


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def build():
    old = {i['key']: i for i in read(ROOT/'catalog/releases/8/ingredients.json')['ingredients']}
    current = {i['key']: i for i in read(OUT/'ingredients.json')['ingredients']}
    decisions = read(OUT/'m3d3-decisions.json')
    manifest = read(OUT/'manifest.json')
    audit = read(OUT/'quality-audit.json')
    assert len(old) == 647 and len(current) == 745
    assert all(current[key] == value for key, value in old.items())
    assert not audit['structuralErrors'] and not audit['aliasCollisionCandidates']
    assert len(decisions) == 131 and Counter(d['decision'] for d in decisions) == {'ACCEPT':98,'DEFER':29,'EXISTING':3,'REJECT':1}
    assert manifest['version'] == 9 and not any(s['id']=='foodon' for s in manifest['sources'])
    added = [i for key, i in sorted(current.items()) if key not in old]
    lines = ['# M3D-3 Release 9 quality and decision audit', '', '**98 new canonical identities; 745 total (624 food / 121 beverage), 269 aliases, 50 categories.** All 647 Release-8 ingredient objects and every category object are preserved exactly. No prior source snapshot is rewritten. The 98 additions carry pinned USDA descriptors plus separate project-authored bilingual labels. No factual nutrition, density, allergen or ABV metadata was inferred; no human linguistic approval is claimed.', '', 'The reproducible candidate ledger has **131 decisions**: 98 ACCEPT, 29 DEFER (27 carried from M3D-2B), 3 EXISTING and 1 REJECT. The static audit reports zero structural errors and zero exact alias collisions; it lists possible English-base concept groups for review, which are not proven duplicates. Seven prior terminology/identity items remain unresolved. No aliases or category nodes were added.', '', '## Source decision', '', '- **USDA FoodData Central SR Legacy, April 2018:** pinned CSV archive SHA-256 `b80817294b8850530aaedf2e515c02593b1824f763a0ff356e5c2081643e6fd0`; official CC0/public-domain terms permit offline redistribution and commercial reuse. USDA descriptors and FDC IDs are preserved in `catalog/preparation/m3d3/source-records.json`. Project label curation is separate.', '- **FoodOn:** term-level lineage/attribution and downstream redistribution have not been cleared; no term imported. **Wikidata:** CC0 identity direction considered, but no pinned QID/revision extraction prepared. **Open Food Facts, TürKomp, TheCocktailDB:** excluded under the standing source restrictions. These exclusions are scope/rights decisions, not assertions that every possible record is unsuitable.', '', '## Accepted source-backed identities', '', '| Stable key | Turkish editorial label | English editorial label | Categories | Source |', '| --- | --- | --- | --- | --- |']
    for i in added:
        external = i['key'].removeprefix('usda-sr-')
        lines.append(f"| `{i['key']}` | {i['names']['tr']} | {i['names']['en']} | {', '.join(i['categories'])} | [FDC](https://fdc.nal.usda.gov/food-details/{external}/nutrients) |")
    lines += ['', '## Non-imported candidates', '', '| Source ID | Decision | Reason |', '| --- | --- | --- |']
    for d in decisions:
        if d['decision'] != 'ACCEPT':
            lines.append(f"| [USDA {d['sourceIdentifier']}]({d['sourceUrl']}) | {d['decision']} | {d['reason']} |")
    lines += ['', 'The seven earlier unresolved decisions remain in the historical review artifacts. Automated decisions here are auditable editorial screening, not expert or individual human linguistic verification. The direct category counts, isolated SQLite imports and upgrade measurements are in `docs/CATALOG_M3D3_COVERAGE.md` and `catalog/production/m3d3-sqlite-report.json`.', '']
    provenance = dict(release=9, source='USDA SR Legacy April 2018', sourceLicense='CC0-1.0', sourceArchiveSha256='b80817294b8850530aaedf2e515c02593b1824f763a0ff356e5c2081643e6fd0', sourceVersions=[dict(id=s['id'],version=s['version'],artifactSha256=s['artifactSha256']) for s in manifest['sources']], additions=[dict(key=i['key'],id=i['id'],sourceExternalId=i['key'].removeprefix('usda-sr-')) for i in added], accepted=98, existing=3, deferred=29, rejected=1, foodOnImported=0, independentlyHumanApprovedAdditions=0)
    return {ROOT/'docs/CATALOG_M3D3_QUALITY_AUDIT.md':'\n'.join(lines), OUT/'m3d3-provenance-report.json':json.dumps(provenance,ensure_ascii=False,indent=2)+'\n'}


if __name__ == '__main__':
    for path, content in build().items():
        if '--check' in sys.argv:
            assert path.read_text(encoding='utf-8') == content, f'Stale quality artifact: {path}'
        else:
            path.write_text(content,encoding='utf-8')
    print('Release 9 quality: 98 additions, 29 deferred, 3 existing, 1 rejected; no structural errors or exact alias collisions')
