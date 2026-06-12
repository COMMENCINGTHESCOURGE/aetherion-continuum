#!/usr/bin/env python3
"""
Sprite → GLTF Reconstructor (Reverse pipeline).

Reconstructs a 3D mesh from sprite frames by:
  1. Loading a cleaned sprite frame
  2. Tracing the alpha contour
  3. Fitting cubic bezier curves (reuses logic from sprite_pipeline.py)
  4. Generating a lathe/revolution mesh from the silhouette
  5. Exporting to GLB

Input:
  - chain_refined/{char_name}_full.png (alpha sprite)
  - Or a single frame PNG with transparent background

Output:
  - {char_name}_reconstructed.glb
"""

from __future__ import annotations

import os
import sys
import math
from pathlib import Path
from typing import Sequence

import numpy as np
from PIL import Image


# ── Contour tracing (from sprite_pipeline.py, adapted) ─────────
def _trace_contour(alpha: np.ndarray) -> np.ndarray:
    h, w = alpha.shape
    visited = np.zeros_like(alpha, dtype=bool)
    contours = []
    for y in range(h):
        for x in range(w):
            if alpha[y, x] > 128 and not visited[y, x]:
                contour = _march(alpha, visited, x, y)
                if len(contour) > 10:
                    contours.append(np.array(contour, dtype=np.float32))
    if not contours:
        raise RuntimeError("No closed contour found in sprite alpha")
    largest = max(contours, key=len)
    return largest


def _march(alpha, visited, sx, sy):
    dirs = [(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1),(0,-1),(1,-1)]
    contour = []
    x, y, d = sx, sy, 0
    for _ in range(100000):
        contour.append([x, y])
        visited[y, x] = True
        found = False
        for i in range(8):
            nd = (d + i) % 8
            nx, ny = x + dirs[nd][0], y + dirs[nd][1]
            if 0 <= nx < alpha.shape[1] and 0 <= ny < alpha.shape[0] and alpha[ny, nx] > 128:
                x, y, d = nx, ny, (nd + 6) % 8
                found = True
                break
        if not found or (x == sx and y == sy):
            break
    return contour


def _rdp_simplify(points: np.ndarray, epsilon: float) -> np.ndarray:
    if len(points) < 3:
        return points
    dmax, idx = 0.0, 0
    end = len(points) - 1
    for i in range(1, end):
        d = _point_line_dist(points[i], points[0], points[end])
        if d > dmax:
            dmax, idx = i, i
    if dmax > epsilon:
        left = _rdp_simplify(points[:idx+1], epsilon)
        right = _rdp_simplify(points[idx:], epsilon)
        return np.vstack([left, right[1:]])
    return np.array([points[0], points[end]])


def _point_line_dist(p, a, b) -> float:
    return abs(np.cross(b-a, p-a)) / (np.linalg.norm(b-a) + 1e-10)


def _cubic_bezier_fit(points: np.ndarray, tension: float = 0.5):
    if len(points) < 4:
        return []
    curves = []
    for i in range(0, len(points)-3, 3):
        p0, p3 = points[i], points[i+3]
        v = (p3 - p0) * tension
        p1 = p0 + v * 0.33
        p2 = p3 - v * 0.33
        curves.append([p0.tolist(), p1.tolist(), p2.tolist(), p3.tolist()])
    return curves


def _sample_bezier(curves: Sequence, samples: int = 64) -> np.ndarray:
    pts = []
    for seg in curves:
        p0, p1, p2, p3 = [np.array(p) for p in seg]
        for i in range(samples):
            t = i / samples
            t1 = 1.0 - t
            pt = (t1**3)*p0 + 3*(t1**2)*t*p1 + 3*t1*(t**2)*p2 + (t**3)*p3
            pts.append(pt)
    return np.array(pts, dtype=np.float32)


# ── Lathe mesh generation from silhouette ──────────────────────
def _make_lathe_mesh(silhouette: np.ndarray, segments: int = 32, depth: float = 0.3) -> tuple[np.ndarray, np.ndarray]:
    """Revolve a 2D silhouette around Y axis to create a vase/urn shape."""
    n = len(silhouette)
    verts = []
    faces = []

    # silhouette: Nx2 array of (x, y) in image space
    # Normalize to unit radius centered at origin
    xs = silhouette[:, 0]
    ys = silhouette[:, 1]
    cx, cy = xs.mean(), ys.mean()
    radius = np.sqrt((xs - cx)**2 + (ys - cy)**2)
    max_r = radius.max()
    if max_r < 1e-6:
        max_r = 1.0
    norm_r = radius / max_r
    norm_y = (ys - cy) / (max_r + 1e-10)

    # Create lathe grid
    for j in range(segments + 1):
        theta = (j / segments) * 2.0 * math.pi
        cos_t, sin_t = math.cos(theta), math.sin(theta)
        for i in range(n):
            r = norm_r[i]
            y = norm_y[i] * depth
            x = r * cos_t
            z = r * sin_t
            verts.append([x, y, z])

    verts = np.array(verts, dtype=np.float32)

    for j in range(segments):
        for i in range(n - 1):
            a = j * n + i
            b = (j + 1) * n + i
            c = (j + 1) * n + (i + 1)
            d = j * n + (i + 1)
            faces.append([a, b, d])
            faces.append([b, c, d])

    return verts, np.array(faces, dtype=np.int32)


def _write_glb(verts: np.ndarray, faces: np.ndarray, out_path: str):
    """Minimal GLB writer (binary)."""
    import struct
    from array import array

    # Build tiny glTF JSON
    accessors = []
    buf_views = []
    meshes = []
    primitives = []

    # Positions
    pos_comp = 5126  # FLOAT
    pos_type = "VEC3"
    pos_bv = 0
    accessors.append({
        "bufferView": pos_bv,
        "componentType": pos_comp,
        "count": len(verts),
        "type": pos_type,
        "min": verts.min(axis=0).tolist(),
        "max": verts.max(axis=0).tolist(),
    })

    # Indices
    idx_bv = 1
    idx_comp = 5123  # UNSIGNED_SHORT if small
    idx_type = "SCALAR"
    idx_count = len(faces) * 3
    accessors.append({
        "bufferView": idx_bv,
        "componentType": idx_comp,
        "count": idx_count,
        "type": idx_type,
    })

    primitives.append({
        "attributes": {"POSITION": 0},
        "indices": 1,
        "mode": 4,
    })
    meshes.append({"primitives": primitives})

    gltf = {
        "asset": {"version": "2.0", "generator": "gltf_from_sprites"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0}],
        "meshes": meshes,
        "accessors": accessors,
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": verts.nbytes, "target": 34962},
            {"buffer": 0, "byteOffset": verts.nbytes, "byteLength": faces.nbytes, "target": 34963},
        ],
    }

    import json
    glb = bytearray()
    json_str = json.dumps(gltf, separators=(",", ":")).encode("utf-8")
    json_pad = (4 - (len(json_str) % 4)) % 4
    json_blob = json_str + b" " * json_pad
    bin_blob = verts.tobytes() + faces.tobytes()
    bin_pad = (4 - (len(bin_blob) % 4)) % 4
    bin_blob = bin_blob + b"\x00" * bin_pad

    # Chunks
    def _chunk(data_bytes, magic):
        length = len(data_bytes)
        header = struct.pack("<I", length) + magic
        return header + data_bytes

    glb.extend(_chunk(b"\x1e\x00\x00\x00" + b"JSON" + json_blob, b"JSON"))
    glb.extend(_chunk(len(bin_blob).to_bytes(4, 'little') + b"BIN\0" + bin_blob, b"BIN\0"))

    Path(out_path).write_bytes(glb)


# ── Pipeline entrypoint ────────────────────────────────────────
def reconstruct_from_sprite(sprite_path: str, out_path: str, depth: float = 0.3, segments: int = 32) -> str:
    img = Image.open(sprite_path).convert("RGBA")
    alpha = np.array(img.split()[-1])

    # Pad alpha to ensure closed contour at edges
    alpha = np.pad(alpha, 5, mode='constant', constant_values=0)

    raw = _trace_contour(alpha)
    simplified = _rdp_simplify(raw, epsilon=2.0)
    curves = _cubic_bezier_fit(simplified, tension=0.6)
    if not curves:
        raise RuntimeError("Bezier fit failed")

    silhouette = _sample_bezier(curves, samples=64)
    verts, faces = _make_lathe_mesh(silhouette, segments=segments, depth=depth)
    _write_glb(verts, faces, out_path)
    return out_path


def main():
    if len(sys.argv) < 2:
        print("Usage: reconstruct_gltf_from_sprites.py <sprite.png> [out.glb]")
        sys.exit(1)
    sprite = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else str(Path(sprite).with_suffix("")) + "_reconstructed.glb"
    reconstruct_from_sprite(sprite, out)
    print(f"Wrote {out}")


if __name__ == "__main__":
    main()
