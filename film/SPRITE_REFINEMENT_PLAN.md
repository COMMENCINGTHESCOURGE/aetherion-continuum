#   9 chars : 6 anims : 12 variants : 5 phases
#   the sprite is the seed. refine the seed and every branch improves.
#   delete one bar, the pipeline breaks.

→ `sprite_bars.json`   — phases 1–4 measurement bars
→ `tripo_bars.json`    — phase 5 measurement bars: local_preview : tripo_production
→ `sprite_pipeline.py` — phases 1–4 engine
→ `phase5_tripo.py`    — phase 5 engine: route → submit → poll → download

```
Phase 1 (clean):     alpha | checker | edge_radius → chain_refined/
Phase 2 (bezier):     rdp_epsilon | bezier_tension → bezier_paths/
Phase 3 (animate):    frame_count | cycle_time | easing → animated_frames/
Phase 4 (variant):    hue_target | sat_scale | val_scale | opacity → variant_sheets/
Phase 5 (tripo):      model_version | face_limit | pbr | format → game_ready.glb
```

`¯` redefines: raw → clean → bezier → animated → variant → game_ready
`/` discards: checkerboard, cropped border, smoothed edge, heightmap vertices, polling latency

route table:
  preview        → local
  iterate        → local
  final_mesh     → tripo
  textured       → tripo
  rigged         → tripo
  variant        → local
  sprite_sheet   → local
  game_ready     → tripo
