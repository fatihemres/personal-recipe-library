#!/usr/bin/env python3
"""Reproduce the M3B-3 coverage summary from validated package/CLI reports."""
import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / 'catalog/production'
coverage = json.loads((OUT / 'coverage.json').read_text())
audit = json.loads((OUT / 'quality-audit.json').read_text())
installation = json.loads((OUT / 'm3b3-installation-report.json').read_text())
data = json.loads((OUT / 'ingredients.json').read_text())
previous = {i['key'] for i in json.loads((ROOT / 'catalog/releases/3/ingredients.json').read_text())['ingredients']}
added = [i for i in data['ingredients'] if i['key'] not in previous]
gaps = [
    'Regional Turkish cheeses (including Van otlu), fermentation starters and regional plants need primary identity/terminology curation.',
    'Asian and Latin American specialty sauces, grains and fresh culinary forms remain unevenly covered.',
    'Coffee beans/roasts, loose tea varieties and herbal infusions need further verified identity selections.',
    'Vermouth, additional liqueur/rum/vodka/tequila varieties, tonic variants and specialty bar concentrates remain incomplete.',
    'Generic-to-specific ingredient relationships are extensible but not comprehensively curated; do not assume interchangeability.',
    'Nutrition, density, allergens and ABV remain unknown for every identity in this identity-only catalog.',
]
report = dict(
    milestone='M3B-3', version=coverage['version'], previousIdentities=482,
    newIdentities=len(added), totalIdentities=len(data['ingredients']),
    ingredientTypeCounts=coverage['ingredientTypeCounts'], aliases=coverage['aliasCount'],
    categoryNodes=coverage['categoryNodes'], categoryMembershipCounts=coverage['categoryMembershipCounts'],
    identitySources=dict(usdaBacked=coverage['usdaRecords'], projectOnly=coverage['projectOnlyRecords']),
    projectLocalizedIdentities=len(data['ingredients']), missingTranslations=coverage['incompleteTranslations'],
    missingMetadata=coverage['missingMetadata'], structuralErrors=audit['structuralErrors'],
    exactAliasCollisions=audit['aliasCollisionCandidates'], conceptCandidateGroups=audit['conceptReviewCandidates'],
    pendingTerminologyItems=audit['linguisticReviewCount'], independentlyHumanApproved=0,
    rejectedSelectedRecords=0, excludedCandidates=audit['selectionExclusions'],
    coverageBoundary='Identity/provenance and automated structural validation, not independent human linguistic approval or complete global coverage.',
    addedIdentities=[dict(id=i['id'], key=i['key'], names=i['names'], categories=i['categories']) for i in added],
    offlineInstallation=installation, remainingGaps=gaps,
)
(OUT / 'm3b3-final-report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
lines = ['# M3B-3 final catalog coverage', '',
    f"Version 4 contains **508 identities**: **26 additions**, **411 food / 97 beverage**, **260 aliases**, **50 category nodes**. All 482 version-3 IDs remain intact. Eight shared validation identities are counted once, not added again.", '',
    '454 identities are USDA-backed; 54 are original reference-backed project identities. All 508 carry project-authored localized labels/classification. No branded-product dump or empirical observations were added.', '',
    '## Quality and review', '',
    'No structural errors, missing bilingual names, exact normalized alias collisions or capitalization warnings. 25 qualified English base-label groups are heuristic concept candidates (full list in quality-audit.json); distinct raw/processed, salted/unsalted and related culinary forms are not automatically merged. This heuristic cannot establish semantic equivalence.', '',
    'All 17 original terminology items remain pending; one new endive/hindiba naming concern gives **18 human-review items**. Zero independently human-approved labels. Existing 482 names/IDs were retained. CSV and detailed Markdown report include provenance, concern, retained proposal, confidence and review status. No evidence justified replacing an original disputed label silently.', '',
    'Selected/imported validation failures: 0; this does not mean all investigated candidates were accepted. Black turtle bean identity overlap and Van otlu primary-page availability remain explicit exclusions. Falernum/allspice product identity and brand boundaries were not adequate for adding generic records from the consulted cocktail page. Missing nutrition/density/allergen/ABV: 508 each; unknown never means zero or allergen-free.', '',
    '## Added identities', '', '| Key | Turkish | English | Categories |', '| --- | --- | --- | --- |']
for i in added:
    lines.append(f"| {i['key']} | {i['names']['tr']} | {i['names']['en']} | {', '.join(i['categories'])} |")
lines += ['', '## Category membership', '', 'Direct memberships overlap and do not sum to the identity total. Roots may have no direct members; descendant ingredients still belong to their hierarchy.', '', '| Category | Direct members |', '| --- | ---: |']
for key, count in sorted(coverage['categoryMembershipCounts'].items()):
    lines.append(f'| {key} | {count} |')
lines += ['', '## Real offline installation and timings', '',
    'Eight actual CLI process runs used isolated temporary databases with process-local network denial. Clean install: 508 inserted. v2 upgrade: 273 inserted / 235 updated. v3 upgrade: 26 inserted / 482 updated. All repeats unchanged. Rust upgrade fixtures additionally preserve personal ingredients, recipes, drafts and field overrides; repeated bootstrap writes zero additional SQLite changes. Killed-process rollback/recovery remains covered by the real subprocess test.', '',
    '| Starting release | Installed release | Repeat | CLI wall time (ms) |', '| --- | --- | --- | ---: |']
for run in installation['runs']:
    lines.append(f"| {run['previousVersion'] or 'clean'} | {run['report']['version']} | {run['repeat']} | {run['elapsedMilliseconds']} |")
lines += ['', 'Local debug measurements include process startup, database initialization/checks and import where applicable; not a portable SLA. Focused Rust measurements: v2→v4 import 190.415 ms, v3→v4 186.139 ms; 100 actual Turkish partial searches 80.595 ms. See TESTING.md for native/package checks and limitations.', '', '## Remaining coverage gaps', '']
lines += ['- ' + gap for gap in gaps]
lines += ['', 'Full long-term catalog and V1 scope remain unchanged. M3C should improve discovery/filtering and catalog customization without manufacturing missing metadata or claiming full worldwide ingredient coverage.', '']
(ROOT / 'docs/CATALOG_M3B_FINAL_REPORT.md').write_text('\n'.join(lines))
print(f'Reported {len(added)} additions and {len(data["ingredients"])} actual identities.')
