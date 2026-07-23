# AETHERION-CONTINUUM

**Field-Native, Conservation-Enforced, Planet-Scale Simulation Platform**

---

## The 40� Leap

Aetherion-Continuum is a B2B infrastructure platform for climate modelers, digital twin startups, and enterprise simulation environments. 

| Metric | Previous Gen | Aetherion-Continuum |
|--------|-------------|---------------------|
| Simulation Throughput | ~5M voxels/frame | **50M field cells/frame** (dense) ? **200M+** (sparse streaming, coherence-filtered) |
| VRAM Footprint | ~800MB (dense/staging) | **~1.6GB dense** (dual vec4 buffers) ? **~68MB sparse** (octree + coherence cache, <5% active) |
| Host-GPU Sync | 12�24 sync points/frame | **Minimal** (camera pos + meta uniforms only; dispatch fully GPU-driven via indirect) |
| Material Phase Resolution | 3�4 discrete states | **Continuous 8D tensor** (field vec4: ?,f,?,C \| gradient vec4: ?T,?M) + phase diagram constraints |
| Dev Iteration Cycle | Compile?Bake?Test (mins) | **Live WGSL hot-reload** (sub-second, field_tensor kernel only) |

---

## Architecture

```
+---------------------------------------------+
�              8D CONTINUUM TENSOR              �
�  field vec4:  ? (density) � f (phase)        �
�               ? (entanglement) � C (cohesion)�
�  gradient vec4: ?T (temp) � ?M (moisture)    �
+---------------------------------------------+
                   �
    +--------------+--------------+
    ?              ?              ?
+--------+  +------------+  +----------+
� Field  �  � Conservation�  �  Sparse  �
� Tensor �  �  Enforce    �  �  Stream  �
� Update �--�  (e < 1e-5) �--�  (68MB)  �
+--------+  +------------+  +----------+
```

---

## Enterprise Integrations

### Python API (Data Science / Climate Modeling)
```python
import aetherion_continuum

# Compile #field DSL to WGSL natively in Python
with open("climate_model.field") as f:
    wgsl_kernel = aetherion_continuum.compile_dsl(f.read())
```

### Native (Rust/wgpu)
```bash
git clone https://github.com/COMMENCINGTHESCOURGE/aetherion-continuum
cd aetherion-continuum
cargo build --release
```

### Proof Verification
```bash
cargo run -- --verify-proofs proofs/proof_chain_20260601_120000.json
```

---

## File Map

```
aetherion-continuum/
+-- core/
�   +-- field_tensor.wgsl          # 6D continuum compute kernel
�   +-- sparse_stream.wgsl         # GPU-driven octree + coherence prediction
�   +-- conservation_enforce.wgsl  # Mass/energy/momentum correction pass
+-- pipeline/
�   +-- zero_sync_dispatch.rs      # Zero-sync compute?render engine
+-- dsl/
�   +-- field_dsl.rs               # #field DSL ? WGSL compiler
+-- proof/
�   +-- conservation_proof.rs      # CRDT-logged invariant proofs
+-- bridge/
�   +-- manifest.rs                # UE5/Blender export manifest
+-- applications/                  # Consumer demos and workflows
+-- src/
�   +-- lib.rs                     # Library entry point (PyO3 bindings)
�   +-- main.rs                    # CLI entry point
+-- Cargo.toml
+-- README.md
```

---

## Building on Windows (Local Dev)

CI builds on Ubuntu and needs nothing from this section. For local Windows builds:

**Prerequisites**
- Rust via [rustup](https://rustup.rs). Two toolchain options:
  - `stable-x86_64-pc-windows-msvc` (default) — requires **Visual Studio Build Tools** (`link.exe`). Without them, only cached builds work; nothing new can link.
  - `stable-x86_64-pc-windows-gnu` — works without Visual Studio:
    ```bash
    rustup toolchain install stable-x86_64-pc-windows-gnu
    cargo +stable-x86_64-pc-windows-gnu build
    ```

**GNU toolchain caveats (full builds / `cargo test`)**
- Crates that generate import libraries (`windows-sys`, `libloading`, `parking_lot_core`) invoke `dlltool`, which shells out to a GNU assembler (`as`). rustup's bundled MinGW is **link-only** (no `as`, and its `gcc.exe` cannot compile/assemble). Fix: install standalone binutils (MSYS2 package `mingw-w64-x86_64-binutils`) and put it on `PATH`.
- The pyo3 test binary links `python311.dll` at load time — add your Python install dir to `PATH` or the test exe exits with `0xC0000135`.

Example `PATH` additions (this repo's dev machine):
```
C:\Users\dasha\.cargo\gnu-shims          # as.exe + dlltool.exe (binutils 2.46) + DLLs
C:\Users\dasha\AppData\Local\Programs\Python\Python311
```

**Run the CI gate locally**
```bash
cargo +stable-x86_64-pc-windows-gnu fmt --all -- --check
cargo +stable-x86_64-pc-windows-gnu clippy --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-gnu test
```

---

## License

MIT
