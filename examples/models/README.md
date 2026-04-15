# Example Models

Third-party OBJ files used by `crates/happyterminals/examples/model-viewer`.
Each file is included for demonstration only; upstream licenses take precedence.

| File | Origin | Triangles | License Notes |
|------|--------|-----------|---------------|
| `bunny.obj` | Stanford 3D Scanning Repository | ~4968 | Research-use caveat upstream; audit before publication. |
| `cow.obj` | MeshLab export (origin TBD) | ~5804 | Provenance unverified; treat as restricted. |
| `teapot.obj` | Utah teapot — Martin Newell | TBD | Typically public domain. |

See `.eclusa/phases/02.1-obj-mesh-loading/02.1-RESEARCH.md` §"Open Questions #1"
for full provenance discussion. A formal license audit is tracked as
HYG-05-adjacent work, out of scope for Phase 2.1.

## Future shading variety

The viewer currently renders all three models with the single default shading
ramp. Per-model shading (variety across bunny/cow/teapot to showcase different
ramp aesthetics) is tracked by the multi-strategy shading backlog entry at
`.eclusa/backlog/999.x-shading-ramp-strategies.md`, which will also introduce
named ramp presets (voxcii / dotted / blocks / braille-density / classic-ascii)
and a DSL-level `.shading()` builder method.

## Replacement candidates

If any file's license blocks future publication, candidates:

- Blender's default Suzanne head (free, in-repo in Blender source).
- Keenan Crane's Spot the cow (MIT-compatible terms per author's page).
