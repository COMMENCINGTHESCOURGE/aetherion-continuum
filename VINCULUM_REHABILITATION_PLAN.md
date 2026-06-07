# VINCULUM REHABILITATION PLAN
## aetherion-continuum wgpu 22 Migration — Substrate Field Manual

---

### PREAMBLE: READING THE SEDIMENT

The substrate has shifted. The wgpu layer beneath your simulation has undergone a **pressure inversion** — what was once solid ground (wgpu 21) has become fluid (wgpu 22). The build fails at 9 fracture points. This manual translates each fracture into substrate metaphors so you can reason through the repair without memorizing API changelogs.

**Metaphor Key:**
- **Pressure** = API requirements / constraints
- **Flow** = data movement through pipelines
- **Binding** = resource connections (buffers, bind groups, layouts)
- **Oxidation** = deprecated/removed APIs corroding away
- **Sediment** = accumulated type changes that settle in new shapes

---

## FRACTURE 1: THE MISSING FEATURE FLAG
### "SHADER_FLOAT_ATOMICS feature not found in wgpu::Features"

**Substrate Reading:**
The feature flag `SHADER_FLOAT_ATOMICS` has **oxidized** — it no longer exists as a discrete feature in wgpu 22. The capability has been **absorbed into the substrate**; float atomics are now baseline on supporting hardware, or gated differently.

**What the Pressure Looks Like:**
```rust
// Line 630 in zero_sync_dispatch.rs — the old pressure point
required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
    | wgpu::Features::INDIRECT_FIRST_INSTANCE
    | wgpu::Features::SHADER_FLOAT_ATOMICS,  // <-- This feature flag dissolved
```

**Field Repair Steps:**
1. **Remove the flag** — Delete `| wgpu::Features::SHADER_FLOAT_ATOMICS` from the feature mask.
2. **Verify hardware support** — The capability now lives in `wgpu::Features::SHADER_FLOAT_ATOMICS` (re-added in wgpu 22.1+) OR is implicitly available when `device.limits().max_storage_buffer_binding_size` is sufficient. Check the wgpu 22 release notes: float atomics moved to `Features::SHADER_FLOAT_ATOMICS` but may require a newer wgpu version.
3. **Test path** — If your shaders use `atomicAdd` on `f32` in storage buffers, the shader will fail to compile at runtime if the hardware doesn't support it. The feature flag was a *compile-time gate*; now it's a *runtime capability check*.

**Layman's Rule:** *Don't ask for a feature that's become air. Breathe it instead.*