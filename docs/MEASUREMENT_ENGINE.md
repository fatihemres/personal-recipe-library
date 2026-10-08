# Measurement engine specification

## Implemented foundation (M2A)

Authoritative servings and line quantities are decimal text with 1–12 integer digits and optional 1–6 fractional digits, no sign/exponent, no leading zeros except 0.x, and value greater than zero. Fractions such as 0.5 are allowed for count units; no automatic whole-count rounding. User comma input becomes a decimal dot without numeric parsing. Trailing zeros are preserved. Unknown/as-needed line quantity is NULL; servings is required. Precision/range violations are rejected, never silently rounded. Original unit selection is a stable database reference.

| Code | Dimension | Canonical unit | Exact factor |
| --- | --- | --- | --- |
| g | mass | g | 1 |
| kg | mass | g | 1000 |
| mL | volume | mL | 1 |
| cc | volume | mL | 1 |
| L | volume | mL | 1000 |
| adet | count | adet | 1 |

This metadata establishes cc=mL. M2A does not calculate conversions, scaling, normalized quantity fields, density conversions or alcohol estimates. No mL=g assumption. The text/UI truthfully states that units are not automatically converted.

## Required full V1 engine (M6, unchanged scope)

Deliver every mass/volume/count/culinary/bar/temperature unit and configurable household measures in MASTER_SPEC R09, including personal presets (25/35/50 cc/custom), preferred display units, shot/jigger/dash/bar spoon, US fluid ounces vs mass ounces, Celsius/Fahrenheit, pinches/drops/packets/slices and custom units. Definitions must preserve historical contexts and exact rational factors.

Persist canonical normalized and original display quantities separately through additive migrations; preserve original entered values. Use exact decimal/rational arithmetic, explicit rounding at display boundaries and sensible fractional-count guidance. Cross-dimension conversion requires valid contextual density with units, provenance/uncertainty and preparation context. Explain unavailable conversions. Scaling preserves unrounded stored quantities. Unknown nutrition, density and ABV remain unknown. Mixer/dilution-aware beverage estimates require enough valid information and visible assumptions; never claim exact final alcohol where dilution or ingredient ABV is unknown.

Acceptance tests must cover exact compatible conversions, cc=mL, ounce dimension distinction, contextual density rejection/acceptance, zero/unknown state, extreme precision, round-trip stability, recipe scaling, count rounding, personal definition snapshots, temperature and cocktail quantity/estimated ABV cases. M2A tests validate decimal syntax/precision and persistence only; they do not establish full-engine completion.
