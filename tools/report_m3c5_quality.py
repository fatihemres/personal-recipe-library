#!/usr/bin/env python3
"""Reproducible release-6 quality and review-decision report."""
import csv
import json
from pathlib import Path
import sys

from apply_final_terminology_decisions import CHANGES, RETAIN, UNRESOLVED

ROOT = Path(__file__).resolve().parents[1]


def build():
    catalog = json.loads((ROOT / "catalog/production/ingredients.json").read_text())
    previous = json.loads((ROOT / "catalog/releases/5/ingredients.json").read_text())
    audit = json.loads((ROOT / "catalog/production/quality-audit.json").read_text())
    manifest = json.loads((ROOT / "catalog/production/manifest.json").read_text())
    with (ROOT / "docs/INGREDIENT_LOCALIZATION_REVIEW.csv").open(newline="", encoding="utf-8") as handle:
        review = list(csv.DictReader(handle))
    now = {row["key"]: row for row in catalog["ingredients"]}
    before = {row["key"]: row for row in previous["ingredients"]}
    assert manifest["version"] == 6 and len(now) == len(before) == 508
    assert len(catalog["categories"]) == 50 and sum(len(i["aliases"]) for i in now.values()) == 260
    assert {x["key"]: x["id"] for x in catalog["ingredients"]} == {x["key"]: x["id"] for x in previous["ingredients"]}
    assert not audit["structuralErrors"] and not audit["aliasCollisionCandidates"]
    assert len(audit["conceptReviewCandidates"]) == 25
    assert len(review) == 18
    assert {x["canonical_key"] for x in review if x["decision"] == "user-approved-name"} == set(CHANGES) | {
        "usda-sr-170918", "usda-sr-168921", "usda-sr-168588"}
    assert {x["canonical_key"] for x in review if x["decision"] == "user-authorized-retain-name"} == set(RETAIN)
    assert {x["canonical_key"] for x in review if x["decision"] == "retain-pending-human-review"} == set(UNRESOLVED)
    for key, row in now.items():
        old = before[key]
        if key in CHANGES:
            identifier, locale, old_name, new_name = CHANGES[key]
            assert row["id"] == identifier and old["names"][locale] == old_name and row["names"][locale] == new_name
            other = "en" if locale == "tr" else "tr"
            assert row["names"][other] == old["names"][other]
            assert {k: v for k, v in row.items() if k not in ("names", "provenance", "canonicalName")} == {k: v for k, v in old.items() if k not in ("names", "provenance", "canonicalName")}
            assert row["canonicalName"] == (new_name if locale == "en" else old["canonicalName"])
            assert row["provenance"][:-1] == old["provenance"]
        else:
            assert row == old, f"Unexpected catalog change: {key}"
    report = dict(catalogVersion=6, identities=508, aliases=260, categoryNodes=50,
                  newlyChangedNames=[dict(key=key, id=value[0], locale=value[1], previous=value[2], selected=value[3]) for key, value in CHANGES.items()],
                  previouslyAppliedNameCount=3, retainedNameKeys=sorted(RETAIN),
                  unresolvedKeys=sorted(UNRESOLVED), independentlyExpertVerified=0,
                  conceptCandidateGroups=25, structuralErrors=[], exactAliasCollisions=[],
                  scope="Two user-authorized label changes; ten user-authorized no-change decisions; three unresolved. AI suggestions are not independent linguistic verification.")
    lines = ["# M3C-5 catalog terminology quality report", "",
             "Release 6 contains **508 stable identities, 260 aliases and 50 categories**. The package changes exactly two canonical display names; it retains all other release-5 ingredient records unchanged. No independent expert linguistic verification is claimed.", "",
             "| Key | Field | Previous | User-authorized value |", "| --- | --- | --- | --- |"]
    for key, (_, locale, old_name, new_name) in CHANGES.items():
        lines.append(f"| `{key}` | {locale} | {old_name} | {new_name} |")
    lines += ["", "Three earlier M3C-4 name choices remain applied. The following ten records retain their current Turkish names under the user's limited implementation decision; their earlier AI suggestions are not independent expert verification:", "",
              "| Key | Retained Turkish name |", "| --- | --- |"]
    lines += [f"| `{key}` | {name} |" for key, name in RETAIN.items()]
    lines += ["", "Three items remain unresolved and unchanged:", "", "| Key | Current Turkish name |", "| --- | --- |"]
    lines += [f"| `{key}` | {name} |" for key, name in UNRESOLVED.items()]
    lines += ["", "The structural audit reports zero structural errors and zero exact alias collisions. Its 25 possible concept groups remain candidate comparisons only. Names, recipe references, user overrides and personal ingredients must be checked through SQLite upgrade tests; this artifact audit alone does not prove runtime behavior.", ""]
    return {
        ROOT / "catalog/production/m3c5-quality-report.json": json.dumps(report, ensure_ascii=False, indent=2) + "\n",
        ROOT / "docs/CATALOG_M3C5_QUALITY_AUDIT.md": "\n".join(lines),
    }


if __name__ == "__main__":
    for path, content in build().items():
        if "--check" in sys.argv:
            assert path.read_text() == content, f"Stale report: {path}"
        else:
            path.write_text(content)
    print("M3C-5 quality: 508 identities, 2 names changed, 10 retained, 3 unresolved.")
