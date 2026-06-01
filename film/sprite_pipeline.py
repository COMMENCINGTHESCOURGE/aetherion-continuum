"""
9 chars : 6 anims : 12 variants : 4 phases
delete one bar, the pipeline breaks.
"""

import json, os, sys
import numpy as np
from PIL import Image, ImageFilter

BARS_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "sprite_bars.json")
SPRITE_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets", "sprites")

def load_bars():
    with open(BARS_PATH) as f:
        return json.load(f)

# ═══ PHASE 1: CLEAN ═══

def clean_sprite(img, bar):
    data = np.array(img).copy()
    data[data[:, :, 3] < bar["alpha_threshold"]] = [0, 0, 0, 0]
    if bar["checker_removal"]:
        data = _remove_checker(data, bar["checker_tolerance"])
    if bar["bg_removal"]:
        data = _remove_bg(data)
    data = _auto_crop(data, padding=8)
    data = _make_square(data)
    data = _smooth_alpha(data, bar["edge_radius"])
    return Image.fromarray(data)

def _remove_checker(data, tol):
    for check in [[204,204,204],[153,153,153],[255,255,255],[238,238,238]]:
        diff = np.abs(data[:,:,:3].astype(np.int16) - np.array(check, dtype=np.int16))
        data[np.all(diff < tol, axis=2)] = [0,0,0,0]
    return data

def _remove_bg(data):
    bg = data[0,0,:3].astype(np.int16)
    diff = np.abs(data[:,:,:3].astype(np.int16) - bg)
    data[np.all(diff < 30, axis=2)] = [0,0,0,0]
    return data

def _auto_crop(data, padding):
    alpha = data[:,:,3]
    rows = np.any(alpha > 10, axis=1)
    cols = np.any(alpha > 10, axis=0)
    if not rows.any() or not cols.any():
        return data
    r0, r1 = np.where(rows)[0][[0,-1]]
    c0, c1 = np.where(cols)[0][[0,-1]]
    r0, r1 = max(0,r0-padding), min(data.shape[0],r1+padding+1)
    c0, c1 = max(0,c0-padding), min(data.shape[1],c1+padding+1)
    return data[r0:r1, c0:c1]

def _make_square(data):
    h, w = data.shape[:2]
    s = max(h, w)
    out = np.zeros((s, s, 4), dtype=data.dtype)
    out[(s-h)//2:(s-h)//2+h, (s-w)//2:(s-w)//2+w] = data
    return out

def _smooth_alpha(data, radius):
    if radius <= 0: return data
    alpha = Image.fromarray(data[:,:,3])
    alpha = alpha.filter(ImageFilter.GaussianBlur(radius=radius))
    data[:,:,3] = np.array(alpha)
    data[data[:,:,3] < 8] = [0,0,0,0]
    return data

# ═══ PHASE 2: BEZIER ═══

def bezier_fit(img, bar):
    data = np.array(img)
    alpha = data[:,:,3]
    binary = (alpha > 128).astype(np.uint8) * 255
    contours = _trace_contours(binary)
    if not contours:
        return None
    contour = max(contours, key=len)
    simplified = _rdp_simplify(contour, bar["rdp_epsilon"])
    curves = _cubic_bezier_fit(simplified, bar["bezier_tension"])
    return {"contour": contour.tolist(), "simplified": simplified.tolist(), "curves": curves}

def _trace_contours(binary):
    contours = []
    visited = np.zeros_like(binary, dtype=bool)
    h, w = binary.shape
    for y in range(h):
        for x in range(w):
            if binary[y,x] > 0 and not visited[y,x]:
                contour = _trace_one(binary, visited, x, y)
                if len(contour) > 10:
                    contours.append(np.array(contour))
    return contours

def _trace_one(binary, visited, sx, sy):
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
            if 0 <= nx < binary.shape[1] and 0 <= ny < binary.shape[0] and binary[ny,nx] > 0:
                x, y, d = nx, ny, (nd + 6) % 8
                found = True
                break
        if not found or (x == sx and y == sy):
            break
    return contour

def _rdp_simplify(points, epsilon):
    if len(points) < 3:
        return points
    dmax, idx = 0, 0
    end = len(points) - 1
    for i in range(1, end):
        d = _point_line_dist(points[i], points[0], points[end])
        if d > dmax:
            dmax, idx = d, i
    if dmax > epsilon:
        return np.vstack([_rdp_simplify(points[:idx+1], epsilon),
                          _rdp_simplify(points[idx:], epsilon)[1:]])
    return np.array([points[0], points[end]])

def _point_line_dist(p, a, b):
    return np.abs(np.cross(b-a, p-a)) / np.linalg.norm(b-a)

def _cubic_bezier_fit(points, tension):
    if len(points) < 4:
        return []
    curves = []
    for i in range(0, len(points)-3, 3):
        p0, p3 = points[i], points[i+3]
        v = (p3 - p0) * tension
        p1 = p0 + v * 0.33
        p2 = p3 - v * 0.33
        curves.append({"p0": p0.tolist(), "p1": p1.tolist(), "p2": p2.tolist(), "p3": p3.tolist()})
    return curves

# ═══ PHASE 3: ANIMATE ═══

def generate_animation(bezier_data, anim_bar, size):
    n = anim_bar["frame_count"]
    total_t = anim_bar["cycle_time"]
    easing = _parse_easing_cp(anim_bar["easing"])
    frames = []
    for i in range(n):
        t_raw = i / n
        t = _dilla_swing(t_raw, easing)
        frame = _render_bezier_frame(bezier_data, t, size)
        frames.append(frame)
    return frames

def _parse_easing_cp(flat):
    return [(flat[i], flat[i+1]) for i in range(0, len(flat)-1, 2)]

def _dilla_swing(t, cp):
    if not cp:
        return t
    for i in range(len(cp)-1):
        x0, y0 = cp[i]
        x1, y1 = cp[i+1]
        if x0 <= t <= x1:
            local = (t - x0) / (x1 - x0 + 1e-10)
            return y0 + (y1 - y0) * (local * local * (3 - 2 * local))
    return t

def _render_bezier_frame(bezier_data, t, size):
    img = Image.new("RGBA", (size, size), (0,0,0,0))
    return img

# ═══ PHASE 4: VARIANT ═══

def apply_variant(rgba, variant_bar):
    arr = rgba.copy()
    rgb, alpha = arr[..., :3], arr[..., 3:]
    hsv = _rgb_to_hsv(rgb)
    h, s, v = hsv[..., 0], hsv[..., 1], hsv[..., 2]
    h[:] = h * (1.0 - variant_bar["hue_blend"]) + variant_bar["hue_target"] * variant_bar["hue_blend"]
    s[:] = np.clip(s * variant_bar["sat_scale"], 0, 1)
    v[:] = np.clip(v * variant_bar["val_scale"], 0, 1)
    if "tint" in variant_bar:
        t = variant_bar["tint"]
        rgb_out = _hsv_to_rgb(np.stack([h, s, v], axis=-1))
        rgb_out[..., 0] = np.clip(rgb_out[..., 0] + t[0], 0, 1)
        rgb_out[..., 1] = np.clip(rgb_out[..., 1] + t[1], 0, 1)
        rgb_out[..., 2] = np.clip(rgb_out[..., 2] + t[2], 0, 1)
    else:
        rgb_out = _hsv_to_rgb(np.stack([h, s, v], axis=-1))
    alpha = alpha * variant_bar["opacity"]
    return np.concatenate([rgb_out, alpha], axis=-1)

def _rgb_to_hsv(rgb):
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    maxc = np.maximum(np.maximum(r, g), b)
    minc = np.minimum(np.minimum(r, g), b)
    diff = maxc - minc
    v = maxc
    s = np.where(maxc > 0, diff / (maxc + 1e-10), 0.0)
    h = np.zeros_like(r)
    mask = diff > 1e-10
    rmask = mask & (maxc == r)
    gmask = mask & (maxc == g) & ~rmask
    bmask = mask & (maxc == b) & ~rmask & ~gmask
    h[rmask] = ((g[rmask] - b[rmask]) / diff[rmask]) % 6
    h[gmask] = ((b[gmask] - r[gmask]) / diff[gmask]) + 2
    h[bmask] = ((r[bmask] - g[bmask]) / diff[bmask]) + 4
    h = h / 6.0
    return np.stack([h, s, v], axis=-1)

def _hsv_to_rgb(hsv):
    h, s, v = hsv[..., 0], hsv[..., 1], hsv[..., 2]
    h = (h % 1.0) * 6.0
    i = np.floor(h).astype(int) % 6
    f = h - np.floor(h)
    p = v * (1 - s)
    q = v * (1 - s * f)
    t = v * (1 - s * (1 - f))
    rgb = np.zeros(hsv.shape, dtype=np.float32)
    for idx, (rc, gc, bc) in enumerate([(v,t,p),(q,v,p),(p,v,t),(p,q,v),(t,p,v),(v,p,q)]):
        m = (i == idx)
        for ch, val in enumerate([rc, gc, bc]):
            rgb[..., ch][m] = val[m]
    return rgb

# ═══ PIPELINE ═══

CHAR_MAP = {
    "defender": "armored_defender_sprite_sheet.png",
    "grief": "grief_warrior_sprite_sheet.png",
    "mecha_v2": "mecha_entity_alpha_v2.png",
    "mecha_pixel": "mecha_entity_alpha_v2_pixel.png",
    "aku": "aku_aku_mask_stylized.png",
    "dim_mak": "dim_mak_fighter_full_sheet.png",
    "geode": "geometric_core_geode_flux.png",
    "kraken": "kraken_game_render.png",
    "void_runner": "void_runner_ship.png",
}

VARIANT_CHAR_MAP = {
    "defender": ["defender_shadow", "defender_crystal", "defender_volcanic"],
    "mecha_pixel": ["mecha_stealth", "mecha_overcharge", "mecha_corrupted"],
    "kraken": ["kraken_deep", "kraken_toxic", "kraken_ghost"],
    "aku": ["aku_void", "aku_nature", "aku_frost"],
}

def run_pipeline(char_key=None, phases=None):
    bars = load_bars()
    if phases is None:
        phases = ["clean", "bezier", "animate", "variant"]
    chars = [char_key] if char_key else list(CHAR_MAP.keys())

    for phase in phases:
        phase_bar = bars["phases"][phase]
        out_dir = os.path.join(SPRITE_DIR, phase_bar["output"])
        os.makedirs(out_dir, exist_ok=True)
        print(f"\nPhase {phase_bar['phase']}: {phase}")

        if phase == "clean":
            for ck in chars:
                fname = CHAR_MAP[ck]
                src = os.path.join(SPRITE_DIR, fname)
                if not os.path.exists(src):
                    print(f"  SKIP {ck}: {fname} not found")
                    continue
                img = Image.open(src).convert("RGBA")
                cleaned = clean_sprite(img, bars["characters"][ck])
                for sz in bars["export_sizes"]:
                    out = cleaned.resize((sz, sz), Image.LANCZOS)
                    out.save(os.path.join(out_dir, f"{ck}_{sz}.png"))
                cleaned.save(os.path.join(out_dir, f"{ck}_full.png"))
                print(f"  {ck}: {img.size} -> {cleaned.size} | {len(bars['export_sizes'])+1} exports")

        elif phase == "bezier":
            for ck in chars:
                clean_file = os.path.join(SPRITE_DIR, "chain_refined", f"{ck}_full.png")
                if not os.path.exists(clean_file):
                    print(f"  SKIP {ck}: no clean output")
                    continue
                img = Image.open(clean_file).convert("RGBA")
                result = bezier_fit(img, bars["characters"][ck])
                if result:
                    bpath = os.path.join(out_dir, f"{ck}_bezier.json")
                    with open(bpath, "w") as f:
                        json.dump(result, f)
                    print(f"  {ck}: {len(result['simplified'])} verts -> {len(result['curves'])} curves")

        elif phase == "animate":
            for ck in chars:
                bezier_file = os.path.join(SPRITE_DIR, "bezier_paths", f"{ck}_bezier.json")
                if not os.path.exists(bezier_file):
                    print(f"  SKIP {ck}: no bezier data")
                    continue
                with open(bezier_file) as f:
                    bezier_data = json.load(f)
                for anim_name, anim_bar in bars["animations"].items():
                    frames = generate_animation(bezier_data, anim_bar, bars["export_sizes"][2])
                    for i, frame in enumerate(frames):
                        frame.save(os.path.join(out_dir, f"{ck}_{anim_name}_{i:02d}.png"))
                    print(f"  {ck}/{anim_name}: {len(frames)} frames @ {anim_bar['cycle_time']}s")

        elif phase == "variant":
            for ck in chars:
                if ck not in VARIANT_CHAR_MAP:
                    continue
                anim_dir = os.path.join(SPRITE_DIR, "animated_frames")
                for vk in VARIANT_CHAR_MAP[ck]:
                    vbar = bars["variants"][vk]
                    frames = [f for f in os.listdir(anim_dir) if f.startswith(f"{ck}_")]
                    for fname in sorted(frames)[:1]:
                        src = os.path.join(anim_dir, fname)
                        img = np.array(Image.open(src).convert("RGBA"), dtype=np.float32) / 255.0
                        result = apply_variant(img, vbar)
                        out_img = Image.fromarray((np.clip(result, 0, 1) * 255).astype(np.uint8))
                        out_img.save(os.path.join(out_dir, fname.replace(f"{ck}_", f"{vk}_")))
                print(f"  {ck}: {len(VARIANT_CHAR_MAP[ck])} variants")

    print(f"\n  pipeline complete.")

if __name__ == "__main__":
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--char", help="Single character key to process")
    ap.add_argument("--phase", choices=["clean","bezier","animate","variant"], nargs="+")
    args = ap.parse_args()
    run_pipeline(args.char, args.phase)
