#!/usr/bin/env python3
"""Reproducible summary of the three reviewed terminology edits in catalog v5."""
import csv
import json
from pathlib import Path
import sys

from apply_terminology_decisions import DECISIONS

ROOT = Path(__file__).resolve().parents[1]


def build():
    catalog = json.loads((ROOT / "catalog/releases/5/ingredients.json").read_text())
    audit = json.loads((ROOT / "catalog/releases/5/quality-audit.json").read_text())
    manifest = json.loads((ROOT / "catalog/releases/5/manifest.json").read_text())
    with (ROOT / "catalog/releases/5/ingredient-localization-review.csv").open(newline="", encoding="utf-8") as handle:
        review = list(csv.DictReader(handle))
    by_key = {row["key"]: row for row in catalog["ingredients"]}
    approved = [row for row in review if row["human_approved"] == "true"]
    pending = [row for row in review if row["human_approved"] == "false"]
    assert manifest["version"] == 5 and len(by_key) == 508
    assert len(approved) == 3 and len(pending) == 15 and len(review) == 18
    assert {row["canonical_key"] for row in approved} == set(DECISIONS)
    assert not audit["structuralErrors"] and not audit["aliasCollisionCandidates"]
    changes = []
    for row in approved:
        key = row["canonical_key"]
        identifier, previous, selected = DECISIONS[key]
        assert (row["stable_id"], row["existing_turkish"], row["selected_turkish"]) == (identifier, previous, selected)
        assert by_key[key]["names"]["tr"] == selected
        changes.append(dict(id=identifier, key=key, previousTr=previous, approvedTr=selected,
                            english=by_key[key]["names"]["en"], decision="user-approved-name"))
    assert all(by_key[row["canonical_key"]]["names"]["tr"] == row["existing_turkish"] for row in pending)
    report = dict(catalogVersion=5, identities=508, aliases=sum(len(row["aliases"]) for row in by_key.values()),
                  categoryNodes=len(catalog["categories"]), approvedNames=changes,
                  pendingIds=[row["stable_id"] for row in pending], pendingCount=15,
                  conceptCandidateGroups=len(audit["conceptReviewCandidates"]),
                  structuralErrors=audit["structuralErrors"], exactAliasCollisions=audit["aliasCollisionCandidates"],
                  scope="Three explicit user naming decisions; no alias, canonical ID, source fact, category, or observation changes. Not independent linguistic certification.")
    lines = ["# M3C-4 catalog terminology quality report", "",
             "Catalog version 5 retains 508 stable identities, 260 aliases and 50 categories. Three Turkish names were selected by the user; 15 other review items remain pending. This is not independent linguistic certification.", "",
             "| Stable ID | Previous Turkish | User-selected Turkish | English identity |", "| --- | --- | --- | --- |"]
    for row in changes:
        lines.append(f"| {row['id']} | {row['previousTr']} | {row['approvedTr']} | {row['english']} |")
    lines += ["", "No old display names were promoted to aliases. Almond butter remains separate from almond paste and marzipan. The 25 possible concept groups remain review candidates, not approved merges.", "",
              "The audit found zero structural errors and zero exact alias collisions. All 15 remaining decision cells in the workbook are blank. Source evidence and historical labels remain in the review CSV and archived version 4 package.", "",
              "Eleven isolated CLI imports/opens with process-local network denial verified fresh, v2, v3 and v4 upgrades plus unchanged repeats. The measured results are in `catalog/production/m3c4-installation-report.json`; no personal application profile was inspected or reset.", ""]
    return {
        ROOT / "catalog/production/m3c4-quality-report.json": json.dumps(report, ensure_ascii=False, indent=2) + "\n",
        ROOT / "docs/CATALOG_M3C4_QUALITY_AUDIT.md": "\n".join(lines),
    }


if __name__ == "__main__":
    for path, content in build().items():
        if "--check" in sys.argv:
            assert path.read_text() == content, f"Stale report: {path}"
        else:
            path.write_text(content)
    print("M3C-4 catalog quality: 508 identities; 3 user-approved names; 15 pending; zero structural errors.")
