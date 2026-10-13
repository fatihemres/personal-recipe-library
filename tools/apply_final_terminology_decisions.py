#!/usr/bin/env python3
"""Reproduce catalog release 6 from immutable release 5 and scoped user decisions.

The two labels below are the only catalog name changes. No network or user
database is accessed; no recommendation is represented as expert verification.
"""
import csv
import hashlib
import json
from pathlib import Path

from apply_terminology_decisions import artifact

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "catalog/releases/5"
OUT = ROOT / "catalog/production"
BASE_MANIFEST_SHA256 = "ffd795f7661beda20ccc07e6cabeb5e0f927b847358c1c256f71fc46c4873a8e"
SOURCE_ID = "recipeatlas-terminology-final"
SOURCE_VERSION = "user-decisions-2"
CHANGES = {
    "usda-sr-174842": ("b824153c-c098-8309-ac77-8538c4fb3c4c", "tr", "Soda (club soda)", "Club soda (kokteyl sodası)"),
    "project-siyez-grain": ("ee71c0f4-6d4f-858c-8ae0-32ccb416ea9e", "en", "Siyez einkorn wheat", "Einkorn wheat (siyez)"),
}
RETAIN = {
    "usda-sr-171329": "Paprika",
    "usda-sr-170927": "Öğütülmüş muskat çiçeği",
    "usda-sr-170936": "Öğütülmüş sater otu",
    "usda-sr-171318": "Kuru Frenk maydanozu",
    "usda-sr-170160": "Badem ezmesi (pastacılık)",
    "usda-sr-170903": "Sade az yağlı Yunan tipi yoğurt",
    "usda-sr-175142": "Levrek (karışık türler, çiğ)",
    "project-salep-drink": "Salep (hazırlanmış içecek)",
    "project-aromatic-bitters": "Aromatik kokteyl bitteri",
    "project-creme-de-cassis": "Crème de cassis",
}
UNRESOLVED = {
    "usda-sr-171319": "Chili tozu",
    "usda-sr-170874": "Fermente az yağlı buttermilk",
    "usda-sr-168412": "Hindiba (endivyen, çiğ)",
}


def read_verified_package():
    raw = (BASE / "manifest.json").read_bytes()
    assert hashlib.sha256(raw).hexdigest() == BASE_MANIFEST_SHA256
    manifest = json.loads(raw)
    assert manifest["version"] == 5 and manifest["purpose"] == "production"
    for entry in [manifest["artifact"], *manifest["evidence"].values()]:
        assert hashlib.sha256((BASE / entry["file"]).read_bytes()).hexdigest() == entry["sha256"]
    return manifest, json.loads((BASE / "ingredients.json").read_text())


def main():
    previous, catalog = read_verified_package()
    current_manifest = OUT / "manifest.json"
    if current_manifest.exists():
        assert json.loads(current_manifest.read_text())["version"] in (5, 6), "Refusing to overwrite another release"
    by_key = {row["key"]: row for row in catalog["ingredients"]}
    assert len(by_key) == len(catalog["ingredients"]) == 508
    assert len(catalog["categories"]) == 50
    assert sum(len(row["aliases"]) for row in by_key.values()) == 260
    assert set(CHANGES).isdisjoint(RETAIN) and set(CHANGES).isdisjoint(UNRESOLVED)
    assert set(RETAIN).isdisjoint(UNRESOLVED)
    records = []
    for key, (identifier, locale, before, after) in CHANGES.items():
        row = by_key[key]
        assert row["id"] == identifier and row["names"][locale] == before
        assert after != before
        row["names"][locale] = after
        if locale == "en":
            assert row["canonicalName"] == before
            row["canonicalName"] = after
        description = json.dumps({"identityId": identifier, "locale": locale,
                                  "previous": before, "selected": after,
                                  "decision": "User-authorized display name; no source fact or alias change."},
                                 ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        row["provenance"].append({"sourceId": SOURCE_ID, "sourceVersion": SOURCE_VERSION,
                                  "externalId": key, "description": description})
        records.append({"externalId": key, "description": description})
    for key, name in {**RETAIN, **UNRESOLVED}.items():
        assert by_key[key]["names"]["tr"] == name
    evidence = artifact(OUT / "terminology-final-records.json", sorted(records, key=lambda r: r["externalId"]))
    catalog_artifact = artifact(OUT / "ingredients.json", catalog)
    manifest = dict(previous)
    manifest["version"] = 6
    manifest["artifact"] = catalog_artifact
    manifest["evidence"] = dict(previous["evidence"], **{SOURCE_ID: evidence})
    manifest["sources"] = [*previous["sources"], {
        "id": SOURCE_ID, "version": SOURCE_VERSION,
        "name": "Tarif Atlası final user-authorized terminology decisions",
        "url": "https://github.com/fatihemres/personal-recipe-library",
        "license": "CC0-1.0",
        "attribution": "Two user-authorized display names; not independent linguistic verification or USDA translations.",
        "retrievedAt": "2026-10-13", "artifactSha256": evidence["sha256"],
    }]
    artifact(OUT / "manifest.json", manifest)
    coverage = json.loads((BASE / "coverage.json").read_text())
    coverage.update(version=6, previousIngredientCount=508, newlyAddedIngredients=0,
                    linguisticReviewCount=3,
                    localizationReview="3 earlier names and 2 current names applied; 10 user-authorized retain-name decisions; 3 unresolved. No independent expert verification.")
    artifact(OUT / "coverage.json", coverage)
    review = json.loads((BASE / "localization-review.json").read_text())
    assert len(review) == 18 and {x["key"] for x in review if not x["humanApproved"]} == set(CHANGES) | set(RETAIN) | set(UNRESOLVED)
    for item in review:
        key = item["key"]
        if key in CHANGES:
            identifier, locale, before, after = CHANGES[key]
            assert item["id"] == identifier and item["existingTr" if locale == "tr" else "existingEn"] == before
            item.update(status="user-approved", decision="user-approved-name",
                        humanReviewNecessary=False, humanApproved=True)
            item["selectedTr" if locale == "tr" else "selectedEn"] = after
        elif key in RETAIN:
            item.update(status="user-retained", decision="user-authorized-retain-name",
                        humanReviewNecessary=False, humanApproved=True)
        elif key in UNRESOLVED:
            assert item["humanReviewNecessary"] and not item["humanApproved"]
    artifact(OUT / "localization-review.json", review)
    with (BASE / "ingredient-localization-review.csv").open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        rows, fields = list(reader), [*reader.fieldnames, "selected_english"]
    assert len(rows) == 18 and len({row["stable_id"] for row in rows}) == 18
    for row in rows:
        key = row["canonical_key"]
        row["selected_english"] = ""
        if key in CHANGES:
            identifier, locale, before, after = CHANGES[key]
            assert row["stable_id"] == identifier and row["existing_turkish" if locale == "tr" else "existing_english"] == before
            row.update(human_review_required="false", human_approved="true", decision="user-approved-name")
            row["selected_turkish" if locale == "tr" else "selected_english"] = after
        elif key in RETAIN:
            assert row["existing_turkish"] == RETAIN[key]
            row.update(human_review_required="false", human_approved="true", decision="user-authorized-retain-name")
        elif key in UNRESOLVED:
            assert row["existing_turkish"] == UNRESOLVED[key]
            assert row["human_approved"] == "false" and row["decision"] == "retain-pending-human-review"
    with (ROOT / "docs/INGREDIENT_LOCALIZATION_REVIEW.csv").open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields, lineterminator="\n", quoting=csv.QUOTE_ALL)
        writer.writeheader()
        writer.writerows(rows)
    print("Prepared catalog v6: 508 identities; 2 labels changed, 10 retained, 3 unresolved.")


if __name__ == "__main__":
    main()
