#!/usr/bin/env python3
"""Reproduce the release-8 editorial audit and source provenance report."""
import json
from collections import Counter
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'catalog/production'


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def build():
    previous=read(ROOT/'catalog/releases/7/ingredients.json')
    current=read(OUT/'ingredients.json')
    manifest=read(OUT/'manifest.json')
    decisions=read(OUT/'m3d2b-decisions.json')
    audit=read(OUT/'quality-audit.json')
    prior=read(ROOT/'catalog/preparation/m3d2a/prior-review.json')
    old={i['key']:i for i in previous['ingredients']}
    new={i['key']:i for i in current['ingredients']}
    added=[i for key,i in sorted(new.items()) if key not in old]
    accepted={d['candidateId']:d for d in decisions if d['decision']=='ACCEPT'}
    for key,value in old.items():
        compared=json.loads(json.dumps(new[key]))
        for provenance in compared['provenance']:
            versions={
                'usda-sr-legacy-m3d1': ('2018-04-selection-5', '2018-04-selection-5-reviewed'),
                'recipeatlas-curation-m3d1': ('production-4', 'production-4-reviewed'),
            }.get(provenance['sourceId'])
            if versions:
                assert provenance['sourceVersion']==versions[1]
                provenance['sourceVersion']=versions[0]
        assert compared==value
    assert len(old)==568 and len(new)==647 and len(added)==len(accepted)==79
    assert audit['auditedIdentities']==647 and not audit['structuralErrors'] and not audit['aliasCollisionCandidates']
    assert len(prior)==7 and all(p['humanDecision'] is None for p in prior)
    assert not any(s['id']=='foodon' for s in manifest['sources'])
    source_counts=dict(sorted(Counter(p['sourceId'] for i in current['ingredients'] for p in i['provenance']).items()))
    provenance=dict(release=8, licenseReview='USDA SR Legacy April 2018 CC0; FoodOn term lineage/attribution unresolved and excluded', sources=manifest['sources'], provenanceLinksBySource=source_counts, acceptedUSDA=[dict(key=i['key'],sourceId='usda-sr-legacy-m3d2b',externalId=i['key'].removeprefix('usda-sr-'),sourceUrl=accepted['m3d2a-usda-'+i['key'].removeprefix('usda-sr-')]['sourceUrl']) for i in added], foodOnImported=0, independentHumanLinguisticApprovals=0)
    counts=Counter(d['decision'] for d in decisions)
    lines=['# M3D-2B release 8 catalog quality and decisions','','**79 new canonical USDA identities; 647 total, 269 aliases, 50 categories.** All 568 release-7 identities, names, categories and factual values remain unchanged. The 60 M3D-1 records use distinct reviewed source-snapshot versions to prevent a collision with a 61-record preview installation. The 79 additions comprise 72 food and 7 beverage identities. No new alias was asserted without equivalence evidence.','','The decision ledger contains **194 evaluated candidates**: USDA 79 ACCEPT and 27 DEFER; FoodOn 84 DEFER for source clearance, three EXISTING/probable concepts and one REJECT brand-specific concept. A FoodOn EXISTING label is a conservative no-import disposition, not a completed cross-source merge. None of the 88 FoodOn terms is in release 8. No individual human linguistic approval is claimed.','','The reproducible static audit reports **zero structural errors and zero exact alias collisions**. It flags qualified English-base concept groups for review; those are not proven duplicates. There are seven unchanged historical terminology/identity questions. Missing nutrition, density, allergen and ABV observations remain unknown.','','## Accepted USDA identities','','| Stable key | Turkish | English source descriptor | Categories | Source |','| --- | --- | --- | --- | --- |']
    for item in added:
        d=accepted['m3d2a-usda-'+item['key'].removeprefix('usda-sr-')]
        lines.append(f"| `{item['key']}` | {item['names']['tr']} | {item['names']['en']} | {', '.join(item['categories'])} | [FDC]({d['sourceUrl']}) |")
    lines+=['','## Deferred USDA decisions','','These remain outside production. The exact candidate, source and collision evidence stays in `catalog/production/m3d2b-decisions.json` and the M3D-2A ledger.','','| Candidate | Proposed Turkish | Reason |','| --- | --- | --- |']
    preparation={r['candidateId']:r for r in read(ROOT/'catalog/preparation/m3d2a/candidates.json')}
    for d in decisions:
        if d['source']=='usda' and d['decision']=='DEFER':
            lines.append(f"| `{d['candidateId']}` | {preparation[d['candidateId']]['proposedTurkishName']} | {d['reason']} |")
    lines+=['','## Other-source exclusions','','- **84 FoodOn terms:** defer pending individual lineage, attribution and redistribution clearance. CC BY 4.0 on the repository alone is insufficient to clear imported terms.','- **Ancho, Pasilla, malt syrup:** probable existing concepts; no new identities or merges.','- **Cambozola:** excluded brand-specific commercial identity.','- **Open Food Facts, TürKomp, TheCocktailDB:** no records selected; previously documented licensing/commercial-use constraints remain.','','## Boundary and next source work','','USDA descriptors are pinned CC0 source facts. Turkish/English presentation and classifications are original project editorial work, not upstream translations. The preparation dataset is not imported wholesale. A future source plan is in CATALOG_M3D2B_SOURCE_PLAN.md. Category counts and real SQLite fresh/upgrade results are in CATALOG_M3D2B_COVERAGE.md.','','Decision totals: '+', '.join(f'{key} {value}' for key,value in sorted(counts.items()))+'.','']
    outputs={OUT/'m3d2b-provenance-report.json':json.dumps(provenance,ensure_ascii=False,indent=2)+'\n',ROOT/'docs/CATALOG_M3D2B_QUALITY_AUDIT.md':'\n'.join(lines)}
    return outputs


if __name__=='__main__':
    for path,content in build().items():
        if '--check' in sys.argv:
            assert path.read_text()==content,f'Stale release-8 report: {path}'
        else:
            path.write_text(content)
    print('M3D-2B quality report: 79 additions, 27 deferred USDA, 88 FoodOn excluded; zero structural errors or exact alias collisions')
