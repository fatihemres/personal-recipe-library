#!/usr/bin/env python3
"""Build production catalog v5 from the immutable v4 package and three user decisions.

No network, source archive, personal database, or USDA descriptor is read here.
"""
import hashlib
import csv
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "catalog/releases/4"
OUT = ROOT / "catalog/production"
DECISIONS = {
    "usda-sr-170918": ("8f3055ee-2f8e-8300-87a4-2755af76b4e6", "Karaman kimyonu tohumu", "Frenk kimyonu tohumu"),
    "usda-sr-168921": ("3d663734-ceaf-8fcf-945f-74298021320a", "Tam taneli mavi mısır unu", "Tam tahıllı mavi mısır unu"),
    "usda-sr-168588": ("6d96fb7b-4669-8b54-ad19-b006e3fb2873", "Tuz eklenmemiş sade badem ezmesi", "Badem püresi (sade, tuz ilavesiz)"),
}
SOURCE_ID = "recipeatlas-terminology"
SOURCE_VERSION = "user-decisions-1"
BASE_MANIFEST_SHA256 = "fdf0cda0aa23005d95c5e052cf178194839163f697e3d0a78765a52694276014"


def encoded(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode()


def artifact(path, value):
    data = encoded(value)
    path.write_bytes(data)
    return {"file": path.name, "sha256": hashlib.sha256(data).hexdigest()}


def main():
    manifest_bytes = (BASE / "manifest.json").read_bytes()
    assert hashlib.sha256(manifest_bytes).hexdigest() == BASE_MANIFEST_SHA256
    old_manifest = json.loads(manifest_bytes)
    assert old_manifest["version"] == 4 and old_manifest["purpose"] == "production"
    old_bytes = (BASE / "ingredients.json").read_bytes()
    assert hashlib.sha256(old_bytes).hexdigest() == old_manifest["artifact"]["sha256"]
    for item in old_manifest["evidence"].values():
        assert hashlib.sha256((BASE / item["file"]).read_bytes()).hexdigest() == item["sha256"]
    catalog = json.loads(old_bytes)
    by_key = {row["key"]: row for row in catalog["ingredients"]}
    assert len(by_key) == len(catalog["ingredients"]) == 508
    records = []
    for key, (identifier, previous, selected) in DECISIONS.items():
        row = by_key[key]
        assert row["id"] == identifier and row["names"]["tr"] == previous
        assert all(alias["name"] != previous for alias in row["aliases"])
        row["names"]["tr"] = selected
        description = json.dumps(
            {"identityId": identifier, "previousTr": previous, "selectedTr": selected,
             "decision": "User-selected Turkish display name; English identity and aliases unchanged."},
            ensure_ascii=False, sort_keys=True, separators=(",", ":"),
        )
        row["provenance"].append({"sourceId": SOURCE_ID, "sourceVersion": SOURCE_VERSION,
                                  "externalId": key, "description": description})
        records.append({"externalId": key, "description": description})
    assert len(records) == 3
    new_evidence = artifact(OUT / "terminology-records.json", sorted(records, key=lambda r: r["externalId"]))
    new_catalog = artifact(OUT / "ingredients.json", catalog)
    manifest = dict(old_manifest)
    manifest["version"] = 5
    manifest["artifact"] = new_catalog
    manifest["evidence"] = dict(old_manifest["evidence"], **{SOURCE_ID: new_evidence})
    manifest["sources"] = [*old_manifest["sources"], {
        "id": SOURCE_ID, "version": SOURCE_VERSION,
        "name": "Tarif Atlası user-selected Turkish terminology",
        "url": "https://github.com/fatihemres/personal-recipe-library",
        "license": "CC0-1.0",
        "attribution": "Three user-selected Turkish display names; not USDA translations or factual observations.",
        "retrievedAt": "2026-10-13", "artifactSha256": new_evidence["sha256"],
    }]
    artifact(OUT / "manifest.json", manifest)
    coverage = json.loads((BASE / "coverage.json").read_text())
    coverage.update(version=5, previousIngredientCount=508, newlyAddedIngredients=0,
                    linguisticReviewCount=15,
                    localizationReview="Three user-selected Turkish names applied; 15 decisions remain pending.")
    artifact(OUT / "coverage.json", coverage)
    review = json.loads((BASE / "localization-review.json").read_text())
    assert len(review) == 18
    for item in review:
        if item["key"] in DECISIONS:
            identifier, previous, selected = DECISIONS[item["key"]]
            assert item["id"] == identifier and item["existingTr"] == previous
            item.update(status="user-approved", decision="user-approved-name",
                        selectedTr=selected, humanReviewNecessary=False, humanApproved=True)
    artifact(OUT / "localization-review.json", review)
    with (BASE / "ingredient-localization-review.csv").open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        rows, fields = list(reader), [*reader.fieldnames, "selected_turkish"]
    assert len(rows) == 18 and len({row["stable_id"] for row in rows}) == 18
    for row in rows:
        row["selected_turkish"] = ""
        if row["canonical_key"] in DECISIONS:
            identifier, previous, selected = DECISIONS[row["canonical_key"]]
            assert row["stable_id"] == identifier and row["existing_turkish"] == previous
            row.update(human_review_required="false", human_approved="true",
                       decision="user-approved-name", selected_turkish=selected)
    with (ROOT / "docs/INGREDIENT_LOCALIZATION_REVIEW.csv").open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields, lineterminator="\n", quoting=csv.QUOTE_ALL)
        writer.writeheader()
        writer.writerows(rows)
    print("Prepared catalog v5: 508 stable identities; 3 Turkish labels changed; 15 pending review items.")


if __name__ == "__main__":
    main()
