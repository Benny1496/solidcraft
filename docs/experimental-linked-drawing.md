# Experimental model-linked XY drawing preview

This is a development-only first slice for testing model-linked 2D manufacturing drawing behavior. It is **not** a Drawing workspace, a persisted sheet, a standards-certified projection, an HLR engine, or a replacement for the published drawing roadmap (#860). All work is isolated on the fork's `feature/browser-automation` branch.

## First use

Create or open a 3D plate and run this command through the browser/CLI/engine command API:

```json
{"command":"drawing.preview","params":{"body":"Plate"}}
```

The reply contains `{status:"created",requires_confirmation:false,drawing}`.
`drawing` holds:

- A versioned `solidcraft-drawing-preview/1` record.
- A **bounding rectangle envelope** of the measured 3D body (not a certified silhouette).
- Measured XY circles from vertical cylindrical B-rep faces, with physical diameters; duplicate split cylinder faces are consolidated.
- Overall X and Y dimensions in millimetres.
- An SVG preview with width/height in **physical millimetres** (model dimensions are 1:1 in the SVG coordinate system).
- Positions of the two manually movable overall-dimension labels and an explicit source geometry signature.

The SVG can be displayed by a future drawing viewport or exported by a dedicated adapter. This command does not itself save a file or open a UI window.

## Updating after the model changes

Store the returned `drawing` as a snapshot **outside** the model, then edit the 3D model. Query again with the original drawing:

```json
{"command":"drawing.preview","params":{"body":"Plate","previous":{"format":"solidcraft-drawing-preview/1","body":"Plate","source":{"...":"..."}}}}
```

Pass the **complete** original drawing object from the first reply, not the abbreviated example above.

If the measured source geometry differs, the command returns:

- `status:"stale"`, `requires_confirmation:true`
- `message:"The original 3D model has changed. Update this drawing?"`
- `drawing`: the **unchanged** original drawing
- `proposed`: the new drawing, ready for review

No changes are applied automatically. After the operator confirms, call `drawing.preview` again with that same original `previous` drawing and `accept:true`. The result is `status:"updated"` and its `drawing` contains the measured new geometry. Overall-dimension label positions remain as specified in the previous drawing. With no model change, the command returns `status:"current"`.

**No actual dialog is implemented yet.** The `stale` and `requires_confirmation` values form the backend contract from which the future UI can show the German confirmation window when opening a drawing.

## Supported subset and cautions

- Works on a body with a valid 3D bounding box; models with a different outline (e.g. round, filleted or C-shaped) still show a *bounding envelope*, **not** their true contour.
- Cylinders must be aligned with the Z axis. Side bores currently return an error rather than being misrepresented.
- A cylindrical exterior boss is not automatically classified as a drilled hole; the API deliberately calls the objects `cylinders`.
- Cylindrical faces currently lack sufficient certified provenance for stable hole callouts. We deliberately **do not persist positional hole-number labels** because adding/removing a hole could silently reattach one to the wrong bore. Later work must tie drawing dimensions to stable feature and face identities, with warnings for missing references.
- The measured source signature is rounded to 0.001 mm. Changes smaller than that threshold are outside this prototype's stale-detection guarantee.
- The `previous` snapshot is passed as command input; it is **not yet stored in a `.solidcraft` design**. Saving, revisiting, and acknowledging a drawing is the next integration layer.
- No standard drawing sheet/title block, print tolerance verification, hidden-line removal, associative diameter/centre distance callouts, or 2D drawing workspace exists yet.
- The independent `file.export_view_dxf` browser test is kept isolated because it previously blocked Chrome DevTools for 240 seconds.

## Automated verification

`xtask/src/browser_drawing_preview_test.js` is run with the existing compiled SolidCraft browser application by `cargo xtask web-api-test --site dist/web`.

The test creates a 150 x 80 x 10 mm plate with four Ø8 holes; verifies the SVG/geometry/dimensions; changes the 3D length to 180 mm and a linked bore to X165; requires explicit confirmation; verifies the original snapshot and manually placed overall-dimension labels survive; and ensures that drawing commands do not mutate the CAD timeline. Do not claim success until the new workflow run has passed.
