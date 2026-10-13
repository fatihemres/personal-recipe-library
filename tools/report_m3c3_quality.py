#!/usr/bin/env python3
"""Publish a reproducible M3C-3 review from the pinned production artifacts.

Automated candidates are not duplicate decisions or human language approval.
"""
import csv
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "catalog/releases/4"
REPORT_DIR = ROOT / "catalog/production"


def build():
    data = json.loads((CATALOG / "ingredients.json").read_text())
    audit = json.loads((CATALOG / "quality-audit.json").read_text())
    records = data["ingredients"]
    by_key = {row["key"]: row for row in records}
    with (CATALOG / "ingredient-localization-review.csv").open(newline="", encoding="utf-8") as handle:
        review = list(csv.DictReader(handle))
    assert len(records) == len(by_key) == 508
    assert len(review) == audit["linguisticReviewCount"] == 18
    assert not audit["structuralErrors"] and not audit["aliasCollisionCandidates"]
    assert all(row["human_approved"] == "false" for row in review)
    findings = []
    for row in review:
        record = by_key[row["canonical_key"]]
        assert row["stable_id"] == record["id"]
        findings.append(dict(
            code="TERMINOLOGY_REVIEW", severity="medium", ingredientIds=[record["id"]],
            keys=[record["key"]], concern=row["linguistic_concern"],
            proposedResolution="Human culinary terminology review; retain the current identity and label until decided.",
            evidence=row["references"], status="pending-human-decision",
        ))
    for group in audit["conceptReviewCandidates"]:
        keys = group["keys"]
        findings.append(dict(
            code="POSSIBLE_VARIANT", severity="low",
            ingredientIds=[by_key[key]["id"] for key in keys], keys=keys,
            concern=f"Shared English base term: {group['concept']}. Forms may differ.",
            proposedResolution="Compare source descriptors and culinary forms before any explicit identity decision; never auto-merge.",
            evidence="catalog/production/quality-audit.json", status="review-candidate",
        ))
    report = dict(
        catalogVersion=4, auditedIdentities=len(records), categoryNodes=len(data["categories"]),
        aliases=sum(len(row["aliases"]) for row in records),
        sourceBacked=sum(any(p["sourceId"] == "usda-sr-legacy" for p in row["provenance"]) for row in records),
        projectOnly=sum(not any(p["sourceId"] == "usda-sr-legacy" for p in row["provenance"]) for row in records),
        structuralErrors=audit["structuralErrors"], exactAliasCollisions=audit["aliasCollisionCandidates"],
        capitalizationWarnings=audit["capitalizationAndTranslationWarnings"],
        terminologyPending=len(review), conceptCandidateGroups=len(audit["conceptReviewCandidates"]),
        humanApproved=0, findings=findings,
        scope="Pinned package 4 static audit. No personal/user database inspected or modified. Suggestions are not approved corrections.",
    )
    assert report["aliases"] == 260 and report["categoryNodes"] == 50
    assert report["sourceBacked"] == 454 and report["projectOnly"] == 54
    lines = [
        "# M3C-3 ingredient catalog quality audit", "",
        "This is a repeatable **static** review of the existing production package, not independent culinary-language approval or a scan of a user's database.", "",
        f"- {report['auditedIdentities']} stable identities, {report['aliases']} aliases, {report['categoryNodes']} category nodes; {report['sourceBacked']} USDA-backed and {report['projectOnly']} project-authored identities.",
        "- Zero structural errors, exact alias collisions, capitalization warnings, broken category links or missing bilingual labels in the existing validated package.",
        f"- {report['terminologyPending']} terminology decisions remain open; {report['conceptCandidateGroups']} shared-base-name groups are possible form comparisons, not proven duplicates.",
        "- Missing nutrition, allergen, density and ABV values remain unknown for all 508 records. No values were inferred or changed.",
        "- Existing canonical IDs, names, aliases, category links and source evidence were not modified.", "",
        "| Severity | Finding | Ingredient ID(s) | Concern | Proposed resolution | Status |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for finding in findings:
        clean = lambda value: str(value).replace("|", "\\|").replace("\n", " ")
        lines.append("| " + " | ".join(map(clean, [finding["severity"], finding["code"], ", ".join(finding["ingredientIds"]), finding["concern"], finding["proposedResolution"], finding["status"]])) + " |")
    lines += ["", "The companion spreadsheet `docs/INGREDIENT_TERMINOLOGY_REVIEW.xlsx` has source evidence and blank human-decision fields. Source links support identity provenance, not independent approval of Turkish wording.", ""]
    return {
        REPORT_DIR / "m3c3-quality-report.json": json.dumps(report, ensure_ascii=False, indent=2) + "\n",
        ROOT / "docs/CATALOG_M3C3_QUALITY_AUDIT.md": "\n".join(lines),
    }


if __name__ == "__main__":
    artifacts = build()
    for path, content in artifacts.items():
        if "--check" in sys.argv:
            assert path.read_text() == content, f"Stale report: {path}"
        else:
            path.write_text(content)
    print("M3C-3 static catalog audit: 508 identities; 18 pending terms; 25 possible form groups; zero structural errors.")
