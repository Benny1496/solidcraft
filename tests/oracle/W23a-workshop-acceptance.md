# W23a — workshop plate fixtures and future Drawing acceptance (#4950)

This contribution contains **geometry oracle recipes only**. Each is a reproducible
SolidCraft 3D model with independently calculated body count, volume, surface
area, centre of mass and bounds. Drawing records, persistent references, UI
commands, drawing fonts, HLR and file exporters are **not** implemented here.

Run `python3 tools/oracle-analytic/cases_workshop.py` to regenerate the
analytic `expected.json` values; run `cargo xtask oracle` to replay every
reference case through SolidCraft's CLI. Formula sources and method strings are
in the script and the corresponding expected-value files.

## Exact geometry and states

The W23/M2 checkpoint baseline is a **100 × 80 × 20 mm** plate whose lower-left
model corner is at XY (0, 0). Two independent through-hole features have
identical Ø10 diameters and are 50 mm apart:

| Persistent feature | Model XY centre (mm) | Model diameter (mm) |
| --- | --- | --- |
| `Bore1` (A) | (20, 20) | 10 |
| `Bore2` (B) | (60, 50) | 10 |

The model parameters are `length`, `width`, `thickness`, `bore1_d` and
`bore2_d`; each hole is a separately created feature. The stock's sketch
dimensions drive its length and width, and its extrusion depth is `thickness`.
Each bore also has a construction-circle guide in the stock sketch with a
diameter dimension driven by its corresponding `boreN_d` parameter. The
individual `solid.hole` features use the same parameters and remain separate
CAD operations; construction guides do not cut the stock.
In an actual interactive workflow, the parameter edit is one command, not a
rebuild from an unrelated set of fixed numbers.

| Oracle case | Model state (mm) | What this verifies |
| --- | --- | --- |
| `84-plate-two-bores` | 100 × 80 × 20; `Bore1` Ø10 at (20,20); `Bore2` Ø10 at (60,50) | Baseline; separate references |
| `85-plate-width-100` | 100 × 100 × 20; both bores as in 84 | Only driving `width` 80 → 100 |
| `86-plate-first-bore-12` | 100 × 80 × 20; `Bore1` Ø12; `Bore2` Ø10 | Only `bore1_d` 10 → 12 |
| `87-plate-bore1-deleted` | 100 × 80 × 20; only original `Bore2` Ø10 at (60,50) | Delete `Bore1`, retain `Bore2` identity |
| `88-plate-four-bores` | 150 × 80 × 10; four Ø8 bores at (15,15), (135,15), (15,65), (135,65) | Independent four-hole workshop plate |
| `94-plate-four-rounded-corners` | 100 × 80 × 20; both Ø10 bores; **four vertical R10 exterior corner rounds** | Rounded exterior profile |
| `95-plate-rounded-top-chamfer` | As 94, **plus a 2 mm equal-distance chamfer along the entire tangent-connected upper perimeter** | Rounded corners and chamfered upper edges |

The R10 corner cuts and full upper-perimeter 2 mm chamfer are analytic using
the same quarter-spandrel and tangent-loop perimeter mathematics as oracle
case 33. These revised blends require a fresh full oracle replay before they
can be called verified.

## A3 at 1:2 — geometry versus measured dimension values

For the model in case 84, an A3 landscape drawing is **420 × 297 mm**,
scale **1:2**. Its projected plate envelope on the sheet is **50 × 40 mm**;
each Ø10 hole appears as a Ø5 circle and the 50 mm centre separation projects
to 25 mm. **Annotation values remain in model units**: plate length 100 mm,
plate width 80 mm, `Bore1` Ø10 mm, centre-to-centre 50 mm. Their written
values are not halved by the view scale. Actual text metrics and display
precision follow their eventual upstream owners (#4813 and #4896).

## Acceptance sequence once drawing APIs are available

Every independent edit starts from case 84; the cases in this table are
**reference states**, not a claim that these drawing commands exist today.

| Step / intended command | Resulting oracle state | Expected drawing measurements and labels |
| --- | --- | --- |
| Create geometry from recipe, add diameter on `Bore1`, centre-distance and plate-width dimensions; move labels in paper mm; save and reopen | 84 | Ø10, 50, width 80; every label at its user-chosen paper coordinate |
| `parameters.change` `width` 80 → 100 | 85 | Width becomes 100; Ø10 and centre distance 50 remain; all label positions unchanged |
| `parameters.change` `bore1_d` 10 → 12 | 86 | `Bore1` becomes Ø12; centre distance 50 and width 80 remain; labels unchanged |
| `timeline.delete Bore1` | 87 | `Bore2` remains Ø10 with its original identity. `Bore1` diameter and linked centre-distance dimensions become **lost, kept with their last values**; no silent transfer to `Bore2` |
| `edit.undo` immediately after any individual edit | 84 | Baseline measurements Ø10 / 50 / width 80 and original reference state restored |
| `edit.redo` following that undo | 85, 86, or 87, matching the undone edit | Edited measurements and corresponding lost/valid reference state restored; manually placed labels retain their paper positions |

Both **open-drawing** and **closed-drawing** deferred-update sequences must
be tested when the shared drawing commands land:

1. **Drawing open:** Change a driving parameter. The existing drawing and
   labels stay visibly **out of date until Update**; executing Update obtains
   the new values and retains manual label positions.
2. **Drawing closed:** Save and close the drawing, change the 3D model, then
   reopen it. The saved drawing still appears out of date until Update;
   updating does not reset manual label positions.
3. Repeat both sequences for bore changes, deletion and undo/redo. With a
   lost reference, retain the last measured value explicitly as historical
   instead of reporting it as a current measurement. A replacement bore
   at the same XY and diameter must not silently restore old source identity;
   reference repair is a distinct explicit operation.

The coordinated drawing design allows **automatic and deferred** update
modes (#4892). Do not introduce a separate rejected-update contract here.
Native storage, update commands and undo ownership belong to #3890 and
#4892; annotations and numeric formats to #4896; Drawing UI to #4894;
sheet geometry/export to #4890. Add executable engine, desktop UI, browser
and SVG/DXF/PDF/PNG comparisons only after those APIs are implemented.

## Analytic provenance

All source values use independent closed-form mathematics. For rectangular
plates, volume is `LWT − Σπ(d/2)²T`, and surface area includes exposed
hole walls minus circles on both stock faces. For the four vertical R10
rounds, the section loses `(4−π)R²`; the boundary becomes
`P = 2(L+W)−8R+2πR`. A 2 mm upper tangent-loop chamfer removes
`(c²/2)[P−2πc/3]` of material; its removed volume centroid has
`z=T−c/3`. Surface and first moments follow the same exact construction.
Calculations are regenerated by `tools/oracle-analytic/cases_workshop.py`.
