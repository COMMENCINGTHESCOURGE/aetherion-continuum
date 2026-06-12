#!/usr/bin/env python3
"""
GLTF → Sprite Frames renderer (Forward pipeline output).

Renders a GLTF/GLB character from multiple view angles or animation poses
to isolated RGBA PNGs compatible with sprite_pipeline Phase 2+.

Input:
  - Rigged GLTF/GLB with SkinnedMesh
  - Or unrigged static mesh (renders single pose from multiple angles)

Output:
  - chain_refined/{char_name}_{view}_{frame:03d}.png
  - chain_refined/{char_name}_full.png (first frame)

Dependencies:
  - trimesh + numpy (already available)
  - Optional: pyglet / moderngl for GPU renderer; falls back to trimesh
"""

from __future__ import annotations

import os
import sys
import math
import json
from pathlib import Path
from dataclasses import dataclass

import numpy as np
from PIL import Image


@dataclass(frozen=True)
class RenderConfig:
    char_name: str
    input_path: str
    output_dir: str
    width: int = 512
    height: int = 512
    frames: int = 8
    distance: float = 2.5
    elevation_deg: float = 15.0
    bg_color: tuple[int, int, int, int] = (0, 0, 0, 0)
    light_dir: tuple[float, float, float] = (0.6, 0.8, 0.5)
    ambient: float = 0.35
    diffuse: float = 0.75
    start_hue_shift: float = 0.0


def _load_mesh(path: str):
    """Load mesh with trimesh, fallback to stub."""
    try:
        import trimesh
        scene = trimesh.load(path, force='scene')
        meshes = []
        if isinstance(scene, trimesh.Scene):
            for geom in scene.geometry.values():
                meshes.append(geom)
        elif isinstance(scene, trimesh.Trimesh):
            meshes.append(scene)
        return meshes
    except ImportError:
        raise RuntimeError("trimesh required: pip install trimesh")


def _render_wireframe(mesh, width: int, height: int, light_dir, ambient, diffuse, bg_color):
    """Render a single mesh to RGBA using flat shading approximation."""
    # Get vertices and faces
    if hasattr(mesh, 'vertices') and hasattr(mesh, 'faces'):
        verts = np.array(mesh.vertices, dtype=np.float32)
        faces = np.array(mesh.faces, dtype=np.int32)
    else:
        raise RuntimeError("Mesh object missing vertices/faces")

    normals = mesh.vertex_normals if hasattr(mesh, 'vertex_normals') and mesh.vertex_normals is not None else _compute_vertex_normals(verts, faces)

    # Simple orthographic projection centered on bbox
    mins = verts.min(axis=0)
    maxs = verts.max(axis=0)
    center = (mins + maxs) * 0.5
    scale = max(maxs - mins) * 0.6
    if scale < 1e-6:
        scale = 1.0

    # Project to NDC [-1, 1]^2
    v = (verts - center) / scale
    ndc = v[:, :2]  # drop z for ortho

    # Rasterize simple
    img = np.full((height, width, 4), dtype=np.uint8, fill_value=list(bg_color))
    light = np.array(light_dir, dtype=np.float32)
    light /= (np.linalg.norm(light) + 1e-10)

    # Painter's sort by face centroid z
    centroids = verts[faces].mean(axis=1)
    order = np.argsort(-centroids[:, 2])

    for fi in order:
        tri = faces[fi]
        pts = ndc[tri]  # 3x2
        # Convert to pixel coords
        px = np.stack([
            (pts[:, 0] * 0.5 + 0.5) * (width - 1),
            (1.0 - (pts[:, 1] * 0.5 + 0.5)) * (height - 1),
        ], axis=1)

        # Bounding box
        x0 = max(0, int(px[:, 0].min()))
        x1 = min(width - 1, int(px[:, 0].max()))
        y0 = max(0, int(px[:, 1].min()))
        y1 = min(height - 1, int(px[:, 1].max()))

        if x1 < x0 or y1 < y0:
            continue

        yy, xx = np.mgrid[y0:y1+1, x0:x1+1]
        px_pixels = np.stack([xx, yy], axis=-1).astype(np.float32)

        # Barycentric
        v0, v1, v2 = px_pixels, px[0], px[1], px[2]
        # Compute areas
        area = 0.5 * np.abs(np.cross(px[1]-px[0], px[2]-px[0])[0])
        if area < 0.5:
            continue

        # barycentric coords for each pixel
        def _bary(p):
            v2p = p - px[0]
            v21 = px[2] - px[0]
            v20 = px[1] - px[0]
            dot22 = np.dot(v21, v21)
            dot20 = np.dot(v21, v20)
            dot2p = np.dot(v21, v2p)
            dot00 = np.dot(v20, v20)
            dot0p = np.dot(v20, v2p)
            denom = dot00 * dot22 - dot20 * dot20
            if abs(denom) < 1e-8:
                return np.zeros(p.shape[:2])
            v = (dot22 * dot0p - dot20 * dot2p) / denom
            w = (dot00 * dot2p - dot20 * dot0p) / denom
            u = 1.0 - v - w
            return np.stack([u, v, w], axis=-1)

        bary = _bary(px_pixels)
        mask = (bary >= -1e-3).all(axis=-1)
        if not mask.any():
            continue

        # Interpolate normal
        n = (bary[..., 0:1] * normals[tri[0]] +
             bary[..., 1:2] * normals[tri[1]] +
             bary[..., 2:3] * normals[tri[2]])
        n = n / (np.linalg.norm(n, axis=-1, keepdims=True) + 1e-10)

        # Lambert shading
        lambert = ambient + diffuse * np.clip(np.dot(n, light), 0.0, 1.0)
        lambert = np.clip(lambert, 0.0, 1.0)

        # Base color from mesh face color if available, else terracotta
        if hasattr(mesh, 'visual') and mesh.visual is not None:
            try:
                base = np.array(mesh.visual.face_colors[fi])[:3] / 255.0
            except Exception:
                base = np.array([0.77, 0.55, 0.46])
        else:
            base = np.array([0.77, 0.55, 0.46])

        rgb = (base * lambert[..., None] * 255).astype(np.uint8)
        alpha = np.full_like(rgb[..., :1], 255, dtype=np.uint8)

        # Alpha mask by barycentric
        alpha[~mask] = 0
        rgb[~mask] = 0

        img[y0:y1+1, x0:x1+1, :3] = rgb
        img[y0:y1+1, x0:x1+1, 3:4] = alpha

    return Image.fromarray(img, 'RGBA')


def _compute_vertex_normals(verts, faces):
    normals = np.zeros_like(verts)
    for tri in faces:
        v0, v1, v2 = verts[tri]
        n = np.cross(v1 - v0, v2 - v0)
        normals[tri] += n
    normals /= (np.linalg.norm(normals, axis=-1, keepdims=True) + 1e-10)
    return normals


def render_gltf_frames(config: RenderConfig) -> list[str]:
    meshes = _load_mesh(config.input_path)

    out_dir = Path(config.output_dir) / config.char_name
    out_dir.mkdir(parents=True, exist_ok=True)

    saved = []
    for mesh in meshes:
        for frame_idx in range(config.frames):
            angle_deg = 360.0 * frame_idx / config.frames
            angle_rad = math.radians(angle_deg)
            # Rotate mesh around Y
            if hasattr(mesh, 'vertices'):
                v = np.array(mesh.vertices, dtype=np.float32)
            elif hasattr(mesh, 'vertices'):
                v = np.array(mesh.vertices, dtype=np.float32)
            else:
                continue

            c = v.mean(axis=0)
            v = v - c
            cos_a, sin_a = math.cos(angle_rad), math.sin(angle_rad)
            x = v[:, 0] * cos_a - v[:, 2] * sin_a
            z = v[:, 0] * sin_a + v[:, 2] * cos_a
            v2 = v.copy()
            v2[:, 0] = x
            v2[:, 2] = z
            v2 = v2 + c

            if hasattr(mesh, 'vertices'):
                try:
                    mesh.vertices = v2
                except Exception:
                    pass

            img = _render_wireframe(mesh, config.width, config.height,
                                    config.light_dir, config.ambient, config.diffuse, config.bg_color)
            fname = out_dir / f"{config.char_name}_view_{frame_idx:03d}.png"
            img.save(str(fname))
            saved.append(str(fname))

            if frame_idx == 0:
                full = out_dir / f"{config.char_name}_full.png"
                img.save(str(full))
                saved.append(str(full))
    return saved


def main():
    if len(sys.argv) < 3:
        print("Usage: render_gltf_sprites.py <char_name> <input.gltf/glb> [output_dir]")
        sys.exit(1)

    char_name = sys.argv[1]
    input_path = sys.argv[2]
    output_dir = sys.argv[3] if len(sys.argv) > 3 else str(Path(__file__).parent / "chain_refined")

    cfg = RenderConfig(
        char_name=char_name,
        input_path=input_path,
        output_dir=output_dir,
        frames=8,
        width=512,
        height=512,
    )
    saved = render_gltf_frames(cfg)
    print(f"Rendered {len(saved)} frames -> {output_dir}")


if __name__ == "__main__":
    main()
