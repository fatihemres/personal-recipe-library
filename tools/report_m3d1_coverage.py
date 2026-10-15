#!/usr/bin/env python3
"""Measure release 6 and current production category coverage in real SQLite.

The CLI imports into temporary directories only; no personal application profile
or renderer in-memory catalog is read.
"""
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "src-tauri/target/debug/examples/catalog_import"


def measure(manifest: Path):
    assert CLI.exists(), "Build the catalog_import example first"
    with tempfile.TemporaryDirectory(prefix="recipeatlas-m3d1-coverage-") as name:
        directory = Path(name)
        result = subprocess.run([str(CLI), str(directory), str(manifest)], capture_output=True,
                                text=True, check=True)
        imported = json.loads(result.stdout)
        db_path = directory / "library.sqlite3"
        assert db_path.is_file(), f"Importer did not create {db_path}"
        db = sqlite3.connect(db_path)
        db.row_factory = sqlite3.Row
        categories = [dict(row) for row in db.execute("""
            SELECT c.stable_key AS key, p.stable_key AS parent,
                   c.name_tr AS turkish, c.name_en AS english,
                   count(DISTINCT m.ingredient_id) AS directCount
            FROM ingredient_categories c
            LEFT JOIN ingredient_categories p ON p.id=c.parent_id
            LEFT JOIN catalog_category_memberships m ON m.category_id=c.id
            WHERE c.origin='catalog'
            GROUP BY c.id ORDER BY c.stable_key
        """)]
        types = {row[0]: row[1] for row in db.execute(
            "SELECT ingredient_type,count(*) FROM catalog_ingredients GROUP BY ingredient_type")}
        totals = {table: db.execute(f"SELECT count(*) FROM {table}").fetchone()[0]
                  for table in ("catalog_ingredients", "catalog_aliases", "ingredient_categories")}
        integrity = db.execute("PRAGMA integrity_check").fetchone()[0]
        foreign_keys = len(db.execute("PRAGMA foreign_key_check").fetchall())
        db.close()
    assert integrity == "ok" and foreign_keys == 0
    assert imported["validated"] == totals["catalog_ingredients"]
    return dict(release=imported["version"], imported=imported["inserted"],
                totals=totals, ingredientTypes=types, categories=categories,
                integrity=integrity, foreignKeyViolations=foreign_keys)


def main():
    baseline = measure(ROOT / "catalog/releases/6/manifest.json")
    current = measure(ROOT / "catalog/releases/7/manifest.json")
    output = dict(method="Separate temporary SQLite databases imported through the real Rust catalog_import CLI; direct distinct category memberships, which overlap.",
                  baseline=baseline, current=current)
    target = ROOT / "catalog/production/m3d1-coverage.json"
    target.write_text(json.dumps(output, ensure_ascii=False, indent=2) + "\n")
    lines = ["# M3D-1 catalog coverage assessment", "",
             "Counts below come from isolated SQLite databases populated by the production Rust importer. Category counts are **direct unique membership counts**; an ingredient may belong to several categories, so rows cannot be summed into a catalog total.", "",
             f"Release 6 baseline: **{baseline['totals']['catalog_ingredients']} identities**, **{baseline['totals']['catalog_aliases']} aliases**, **{baseline['totals']['ingredient_categories']} categories**; food {baseline['ingredientTypes'].get('food',0)}, beverage {baseline['ingredientTypes'].get('beverage',0)}.",
             f"Current release {current['release']}: **{current['totals']['catalog_ingredients']} identities**, **{current['totals']['catalog_aliases']} aliases**, **{current['totals']['ingredient_categories']} categories**; food {current['ingredientTypes'].get('food',0)}, beverage {current['ingredientTypes'].get('beverage',0)}.", "",
             "| Category key | Parent | Baseline | Current |", "| --- | --- | ---: | ---: |"]
    old = {c["key"]: c["directCount"] for c in baseline["categories"]}
    for category in current["categories"]:
        lines.append(f"| `{category['key']}` | `{category['parent'] or '—'}` | {old.get(category['key'],0)} | {category['directCount']} |")
    lines += ["", "## Prioritized gaps", "",
              "1. **Cuisine-specific cooking ingredients:** the baseline has only 16 direct `turkish-regional` and 7 `international-specialty` memberships. These labels do not measure all cuisine coverage; South/East/Southeast Asian, Middle Eastern/North African, Mediterranean/European and Latin American ingredients still need separately screened, source-backed candidates. No cuisine-specific category is implied by a name alone.",
              "2. **Beverage preparation:** 1 `bitters`, 3 `mixers`, 4 `coffee`, 6 `tea`, 2 `infusions`, 6 `garnishes`, and 11 `syrups` direct memberships in the baseline make these high-value search gaps. Distinguish ingredient types from prepared drinks and branded products.",
              "3. **Food breadth:** 4 `fermented`, 5 `dairy-alternatives`, 7 `meat`, 7 `poultry`, 8 `mushrooms`, 13 `condiments`, 16 `sauces`, 19 `legumes`, and 21 `seafood` direct memberships indicate useful room for specific forms and proteins. Distinguish raw/cooked/dried/powdered forms only when source descriptors support them.",
              "4. **Taxonomy and translation:** the current 50-node hierarchy is retained in this increment. Existing categories may contain ingredients from several cuisines; a zero cuisine-specific node is not a zero ingredient count. The three M3C-5 terminology questions remain unresolved. Additional doubtful labels belong in a review queue, outside the production seed.", "",
              "The machine-readable report contains the full category table and import/integrity results. Neither category counts nor USDA descriptor availability certify the cultural appropriateness of a translation.", ""]
    (ROOT / "docs/CATALOG_M3D1_COVERAGE_ASSESSMENT.md").write_text("\n".join(lines))
    print(f"Measured SQLite release {baseline['release']}={baseline['totals']['catalog_ingredients']} and release {current['release']}={current['totals']['catalog_ingredients']} identities.")


if __name__ == "__main__":
    main()
