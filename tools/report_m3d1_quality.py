#!/usr/bin/env python3
"""Release-7 boundary audit and reproducible, explicitly unapproved review queue."""
import csv
import hashlib
import json
from pathlib import Path
import sys
from collections import Counter
from prepare_validation_catalog import stable

ROOT = Path(__file__).resolve().parents[1]


def build():
    out = ROOT / 'catalog/production'
    old = json.loads((ROOT / 'catalog/releases/6/ingredients.json').read_text())
    new = json.loads((out / 'ingredients.json').read_text())
    manifest = json.loads((out / 'manifest.json').read_text())
    old_by_key = {i['key']: i for i in old['ingredients']}
    new_by_key = {i['key']: i for i in new['ingredients']}
    assert manifest['version'] == 7 and len(new_by_key) == 568
    assert new['categories'] == old['categories'] and len(new['categories']) == 50
    assert all(new_by_key[key] == item for key, item in old_by_key.items()), 'Historical row changed'
    additions = [i for i in new['ingredients'] if i['key'] not in old_by_key]
    assert len(additions) == 60 and sum(len(i['aliases']) for i in new['ingredients']) == 269
    with (out / 'batches/m3d1-first-curated.tsv').open(encoding='utf-8', newline='') as handle:
        rows = list(csv.DictReader(handle, delimiter='\t'))
    assert {i['key'] for i in additions} == {'usda-sr-'+r['fdc_id'] for r in rows}
    assert len(rows) == 60
    for i in additions:
        assert i['id'] == stable('ingredient', i['key']) and not i['observations']
        assert {p['sourceId'] for p in i['provenance']} == {'usda-sr-legacy-m3d1', 'recipeatlas-curation-m3d1'}
    for name, source in manifest['evidence'].items():
        assert hashlib.sha256((out/source['file']).read_bytes()).hexdigest() == source['sha256']
    assert new_by_key['usda-sr-170283']['names']['tr'] == 'Kabuğu alınmış arpa'
    assert new_by_key['usda-sr-168117']['names']['tr'] == 'Şekersiz turna yemişi suyu'
    assert not any('kara lahana' in a['name'].lower() for a in new_by_key['usda-sr-168421']['aliases'])
    deferred = json.loads((out / 'm3d1-deferred.json').read_text())
    assert len(deferred) == 1 and deferred[0]['candidate']['fdc_id'] == '169406'
    assert 'usda-sr-169406' not in new_by_key
    questions = {
        'usda-sr-170007': 'Welsh onion: Turkish regional naming and distinction from generic green onion need culinary review; no equivalence alias added.',
        'usda-sr-168472': 'Acorn squash: confirm natural Turkish cultivar/type terminology; the source winter-squash identity remains explicit.',
        'usda-sr-174208': 'Northern lobster: confirm preferred Turkish species/common-name terminology; no unsupported species synonym added.',
    }
    review = []
    for key, reason in questions.items():
        i = new_by_key[key]
        review.append(dict(id=i['id'], key=key, tr=i['names']['tr'], en=i['names']['en'], categories=i['categories'],
                           descriptor=i['provenance'][0]['description'], reason=reason, status='terminology-review-needed',
                           includedInRelease=True, proposedCorrection=None, humanDecision=None,
                           sourceUrl=f"https://fdc.nal.usda.gov/food-details/{key.removeprefix('usda-sr-')}/nutrients"))
    held = deferred[0]['candidate']
    review.append(dict(id=stable('ingredient', 'usda-sr-169406'), key='usda-sr-169406', tr=held['tr'], en=held['en'],
                       categories=[held['category']], descriptor=held['expected_description'], reason=deferred[0]['reason'],
                       status='identity-review-needed', includedInRelease=False, proposedCorrection=None, humanDecision=None,
                       sourceUrl='https://fdc.nal.usda.gov/food-details/169406/nutrients'))
    review.sort(key=lambda r: r['key'])
    report = dict(version=7, previousIdentities=508, proposedCandidates=61, newIdentities=60, identities=568,
                  aliases=269, newAliases=9, categoryNodes=50, newCategories=0,
                  ingredientTypeCounts=dict(Counter(i['ingredientType'] for i in new['ingredients'])),
                  newIngredientTypeCounts=dict(Counter(i['ingredientType'] for i in additions)),
                  sourceProvenanceCounts=dict(Counter(p['sourceId'] for i in new['ingredients'] for p in i['provenance'])),
                  newCategoryMemberships=dict(sorted(Counter(c for i in additions for c in i['categories']).items())),
                  deferredIdentities=deferred, newReviewItems=review, historicalUnresolvedTerms=3,
                  independentlyHumanApprovedAdditions=0, missingFactualMetadata={k:568 for k in ['density','nutrition','allergens','abv']},
                  preservation='All 508 complete release-6 ingredient objects and all 50 category objects are unchanged.',
                  additions=additions)
    lines = ['# M3D-1 release 7 quality report', '',
             '**60 additions; 568 identities, 269 aliases, 50 category nodes.** The prepared 61-candidate batch was reduced by one after identity review: unroasted-kernel tahini may overlap existing raw stone-ground tahini. It remains in the deferred queue, with its source descriptor preserved. No historical identity changed.', '',
             'New records: 53 food / 7 beverage. Full catalog: 464 food / 104 beverage. Nine aliases added. All additions use the pinned USDA SR Legacy descriptor and separate project-curated labels. No nutrient, density, allergen or ABV values were inferred. No independent human linguistic approval is claimed.', '',
             'Pre-release corrections: hulled barley is “Kabuğu alınmış arpa”; unsweetened cranberry juice is “Şekersiz turna yemişi suyu”; the kale alias is simply “Kale”, without equating it to Turkish karalahana. These are new candidate corrections, not changes to release-6 records.', '',
             '## Outstanding review', '',
             'The three historical M3C-5 terms remain unchanged and unresolved. Four new review items follow (three included naming questions, one excluded identity question). Blank human decisions are intentional.', '',
             '| Key | Included | Turkish | English | Question |', '| --- | --- | --- | --- | --- |']
    lines += [f"| {r['key']} | {r['includedInRelease']} | {r['tr']} | {r['en']} | {r['reason']} |" for r in review]
    lines += ['', '## Additions', '', '| Stable key | Turkish | English | Categories |', '| --- | --- | --- | --- |']
    lines += [f"| {i['key']} | {i['names']['tr']} | {i['names']['en']} | {', '.join(i['categories'])} |" for i in additions]
    lines += ['', 'Counts are package audit values; actual SQLite counts and overlapping category distribution are in CATALOG_M3D1_COVERAGE_ASSESSMENT.md. Licensing and remaining cuisine/beverage gaps are documented in CATALOG_M3D1_SOURCE_DECISIONS.md. The controlled batch is smaller than the original aspirational target because additional regional and bar ingredients need separate source/identity/translation review.', '']
    return {out/'m3d1-quality-report.json': json.dumps(report, ensure_ascii=False, indent=2)+'\n',
            out/'m3d1-review.json': json.dumps(review, ensure_ascii=False, indent=2)+'\n',
            ROOT/'docs/CATALOG_M3D1_QUALITY_AUDIT.md': '\n'.join(lines)}


if __name__ == '__main__':
    for path, content in build().items():
        if '--check' in sys.argv:
            assert path.read_text() == content, f'Stale M3D-1 report: {path}'
        else:
            path.write_text(content)
    print('M3D-1 audit: 60 additions, 1 deferred, 3 new terminology questions; historical rows unchanged.')
