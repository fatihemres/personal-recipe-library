#!/usr/bin/env python3
"""Build release 7 from immutable release 6 and an explicitly curated USDA selection.

No downloads, automatic translations, nutrition inference or personal DB writes.
"""
import csv
import hashlib
import io
import json
from pathlib import Path
import sys
import unicodedata
import zipfile
from collections import Counter

from prepare_validation_catalog import ARCHIVE_HASH, encoded, stable

ROOT = Path(__file__).resolve().parents[1]
OLD = ROOT / "catalog/releases/6"
OUT = ROOT / "catalog/production"
BATCH = OUT / "batches/m3d1-first-curated.tsv"
OLD_MANIFEST_SHA = "5a8e28efa14308b83b0ad0b3eb4346784c092254577aa8d090a0fb25288b0d32"
USDA_SOURCE = "usda-sr-legacy-m3d1"
CURATION_SOURCE = "recipeatlas-curation-m3d1"
USDA_VERSION = "2018-04-selection-5"
CURATION_VERSION = "production-4"


def norm(locale, value):
    value = unicodedata.normalize("NFC", value)
    if locale == "tr":
        value = value.replace("I", "ı").replace("İ", "i")
    return " ".join(value.lower().split())


def artifact(name, value):
    raw = encoded(value)
    (OUT / name).write_bytes(raw)
    return {"file": name, "sha256": hashlib.sha256(raw).hexdigest()}


def main(archive):
    assert hashlib.sha256((OLD / "manifest.json").read_bytes()).hexdigest() == OLD_MANIFEST_SHA, "Historical release 6 changed"
    old_manifest = json.loads((OLD / "manifest.json").read_text())
    assert old_manifest["version"] == 6
    assert hashlib.sha256((OLD / old_manifest["artifact"]["file"]).read_bytes()).hexdigest() == old_manifest["artifact"]["sha256"], "Historical ingredient artifact changed"
    for evidence in old_manifest["evidence"].values():
        for directory in (OLD, OUT):
            assert hashlib.sha256((directory / evidence["file"]).read_bytes()).hexdigest() == evidence["sha256"], "Historical source evidence changed"
    current = json.loads((OUT / "manifest.json").read_text())
    assert current["version"] in (6, 7), "Refusing to replace an unrelated release"
    old_data = json.loads((OLD / "ingredients.json").read_text())
    current_data = json.loads((OUT / "ingredients.json").read_text())
    if current["version"] == 6:
        assert current_data == old_data, "Working release 6 diverges from archive"
    with BATCH.open(encoding="utf-8", newline="") as handle:
        selected = list(csv.DictReader(handle, delimiter="\t"))
    assert selected and len({row["fdc_id"] for row in selected}) == len(selected)
    assert all(row["fdc_id"].isdigit() and len(row["fdc_id"]) == 6 for row in selected)
    if archive == "--evidence":
        evidence = current["evidence"][USDA_SOURCE]
        raw = (OUT / evidence["file"]).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == evidence["sha256"], "Evidence checksum mismatch"
        facts = {row["externalId"]: row["description"] for row in json.loads(raw)}
    else:
        raw = Path(archive).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == ARCHIVE_HASH, "Pinned USDA archive checksum mismatch"
        with zipfile.ZipFile(io.BytesIO(raw)) as zipped:
            with io.TextIOWrapper(zipped.open("FoodData_Central_sr_legacy_food_csv_2018-04/food.csv"), encoding="utf-8-sig") as handle:
                facts = {row["fdc_id"]: row["description"] for row in csv.DictReader(handle) if row["fdc_id"] in {x["fdc_id"] for x in selected}}
    assert len(facts) == len(selected), "Missing selected USDA descriptor"
    for row in selected:
        assert row["expected_description"] == facts[row["fdc_id"]], f"Descriptor changed: {row['fdc_id']}"

    old_keys = {item["key"] for item in old_data["ingredients"]}
    old_usda_ids = {p["externalId"] for item in old_data["ingredients"] for p in item["provenance"] if p["sourceId"].startswith("usda-sr-legacy")}
    category_keys = {item["key"] for item in old_data["categories"]}
    owners = {}
    for item in old_data["ingredients"]:
        for locale, label in list(item["names"].items()) + [(a["locale"], a["name"]) for a in item["aliases"]]:
            owners[(locale, norm(locale, label))] = item["key"]
    new_items, usda_evidence, curation_evidence = [], [], []
    for row in sorted(selected, key=lambda x: x["fdc_id"]):
        external = row["fdc_id"]
        assert external not in old_usda_ids, f"USDA identity already mapped in release 6: {external}"
        key = f"usda-sr-{external}"
        assert key not in old_keys, f"Already in release 6: {key}"
        categories = [row["category"]] + ([row["extra_categories"]] if row["extra_categories"] else [])
        assert len(categories) == len(set(categories)) and set(categories) <= category_keys
        assert row["preferred_unit"] in ("g", "mL")
        assert row["dimensions"] in ("mass", "volume")
        assert (row["preferred_unit"], row["dimensions"]) in (("g", "mass"), ("mL", "volume"))
        beverage = row["category"] in ("juices", "syrups", "mixers", "soft-drinks", "coffee", "tea", "infusions", "other-beverages", "wines", "beers", "spirits", "liqueurs", "bitters")
        assert (row["preferred_unit"] == "mL") == beverage
        aliases_tr = [name.strip() for name in row["aliases_tr"].split(";") if name.strip()]
        aliases_en = [name.strip() for name in row["aliases_en"].split(";") if name.strip()]
        for locale, label in [("tr", row["tr"]), ("en", row["en"])] + [("tr", a) for a in aliases_tr] + [("en", a) for a in aliases_en]:
            assert label and label == label.strip() and unicodedata.normalize("NFC", label) == label
            owner_key = (locale, norm(locale, label))
            assert owner_key not in owners, f"Name/alias collision: {label} with {owners.get(owner_key)}"
            owners[owner_key] = key
        descriptor = facts[external]
        authored = dict(tr=row["tr"], en=row["en"], aliasesTr=aliases_tr, aliasesEn=aliases_en,
                        categories=categories, preferredUnit=row["preferred_unit"], dimensions=[row["dimensions"]],
                        basis="Project-curated culinary label and classification against pinned USDA description; not an upstream translation or independent linguistic approval")
        authored_description = json.dumps(authored, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        usda_evidence.append(dict(externalId=external, description=descriptor))
        curation_evidence.append(dict(externalId=key, description=authored_description))
        new_items.append(dict(id=stable("ingredient", key), key=key, canonicalName=row["en"],
                              ingredientType="beverage" if beverage else "food",
                              names=dict(tr=row["tr"], en=row["en"]),
                              aliases=[dict(locale="tr", name=a) for a in aliases_tr] + [dict(locale="en", name=a) for a in aliases_en],
                              categories=categories, preferredUnit=row["preferred_unit"], dimensions=[row["dimensions"]], relations=[],
                              provenance=[dict(sourceId=USDA_SOURCE, sourceVersion=USDA_VERSION, externalId=external, description=descriptor),
                                          dict(sourceId=CURATION_SOURCE, sourceVersion=CURATION_VERSION, externalId=key, description=authored_description)],
                              observations=[]))
    ingredient_data = dict(categories=old_data["categories"], ingredients=sorted(old_data["ingredients"] + new_items, key=lambda item: item["key"]))
    # The archive is the immutable baseline; all 508 rows must remain byte-semantic matches.
    assert {i["key"]: i for i in ingredient_data["ingredients"] if i["key"] in old_keys} == {i["key"]: i for i in old_data["ingredients"]}
    usda_artifact = artifact("m3d1-usda-records.json", usda_evidence)
    curation_artifact = artifact("m3d1-curation-records.json", curation_evidence)
    ingredients_artifact = artifact("ingredients.json", ingredient_data)
    sources = old_manifest["sources"] + [
        dict(id=USDA_SOURCE, version=USDA_VERSION, name="USDA SR Legacy April 2018 — M3D-1 selection",
             url="https://fdc.nal.usda.gov/download-datasets/", license="CC0-1.0",
             attribution="USDA Agricultural Research Service, FoodData Central, SR Legacy April 2018; selected descriptors unchanged. Selection version is project-owned.",
             retrievedAt="2026-10-13", artifactSha256=usda_artifact["sha256"]),
        dict(id=CURATION_SOURCE, version=CURATION_VERSION, name="Tarif Atlası M3D-1 curated labels",
             url="https://github.com/fatihemres/personal-recipe-library", license="CC0-1.0",
             attribution="Project-authored bilingual labels and classifications against USDA descriptors; no empirical metadata or independent linguistic approval.",
             retrievedAt="2026-10-13", artifactSha256=curation_artifact["sha256"]),
    ]
    manifest = dict(formatVersion=1, dataset="recipeatlas-ingredients", version=7, purpose="production",
                    artifact=ingredients_artifact,
                    evidence=old_manifest["evidence"] | {USDA_SOURCE: usda_artifact, CURATION_SOURCE: curation_artifact},
                    sources=sources)
    artifact("manifest.json", manifest)
    counts = Counter(category for item in ingredient_data["ingredients"] for category in item["categories"])
    aliases = sum(len(item["aliases"]) for item in ingredient_data["ingredients"])
    coverage = dict(dataset="recipeatlas-ingredients", version=7,
                    previousIngredientCount=len(old_data["ingredients"]), newlyAddedIngredients=len(new_items),
                    selectedRecords=len(ingredient_data["ingredients"]), productionReadyRecords=len(ingredient_data["ingredients"]),
                    productionReadyMeaning="Identity/provenance and importer checks, not independent linguistic approval",
                    ingredientTypeCounts=dict(Counter(item["ingredientType"] for item in ingredient_data["ingredients"])),
                    aliasCount=aliases, categoryNodes=len(category_keys), categoryMembershipCounts=dict(sorted(counts.items())),
                    newSourceRecords=len(new_items), rejectedRecords=0, deferredRecords=1, duplicateSelectedIds=0,
                    incompleteTranslations=0, importedRecords=None,
                    importedRecordsExplanation="Package count only; see SQLite coverage/import report for actual installation",
                    missingMetadata={key: len(ingredient_data["ingredients"]) for key in ("nutrition", "density", "allergens", "abv")},
                    unresolvedHistoricalTerminology=3,
                    newTerminologyReviewCandidates=4,
                    sourceCounts=dict(Counter(p["sourceId"] for item in ingredient_data["ingredients"] for p in item["provenance"])))
    artifact("coverage.json", coverage)
    print(f"Prepared release 7: {len(new_items)} new, {len(ingredient_data['ingredients'])} identities, {aliases} aliases, {len(category_keys)} categories")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: prepare_m3d1_catalog.py /path/to/pinned-USDA-archive.zip")
    main(sys.argv[1])
