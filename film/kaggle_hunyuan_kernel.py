"""
kaggle_hunyuan_kernel.py — runs on Kaggle GPU (T4/P100, 16 GB).
Installs Hunyuan3D-2, loads model, converts image → GLB.
Push via REST API: kernelType=script auto-executes.
Output: /kaggle/working/{char_key}.glb

cost: $0. latency: ~3 min install + ~30s inference.
local_gpu : kaggle_cloud :: zero_dollars : zero_dollars
"""

import os, sys, json, base64, subprocess, time

# ═══ CONFIG ═══
# Set by phase5_kaggle.py before push:
CHAR_KEY = os.environ.get("AETHERION_CHAR_KEY", "defender")
IMAGE_B64 = os.environ.get("AETHERION_IMAGE_B64", "")
OUTPUT_DIR = "/kaggle/working"
MODEL_VERSION = "v2"       # Hunyuan3D-2
USE_TEXTURE = True
FACE_LIMIT = 100000

# ═══ INSTALL (first run only — cached on subsequent) ═══

def install():
    print("[1/4] Installing dependencies...")
    subprocess.run([sys.executable, "-m", "pip", "install", "-q",
        "torch", "torchvision", "torchaudio", "--index-url", "https://download.pytorch.org/whl/cu121"], check=True)
    subprocess.run([sys.executable, "-m", "pip", "install", "-q",
        "trimesh", "Pillow", "numpy", "huggingface_hub", "gradio"], check=True)

    if not os.path.exists("Hunyuan3D-2"):
        print("[2/4] Cloning Hunyuan3D-2...")
        subprocess.run(["git", "clone", "https://github.com/Tencent-Hunyuan/Hunyuan3D-2.git"], check=True)

    os.chdir("Hunyuan3D-2")
    subprocess.run([sys.executable, "-m", "pip", "install", "-q", "-r", "requirements.txt"], check=True)
    subprocess.run([sys.executable, "-m", "pip", "install", "-q", "-e", "."], check=True)

    # Texture renderer
    os.chdir("hy3dgen/texgen/custom_rasterizer")
    subprocess.run([sys.executable, "setup.py", "install"], check=True)
    os.chdir("../../..")
    os.chdir("hy3dgen/texgen/differentiable_renderer")
    subprocess.run([sys.executable, "setup.py", "install"], check=True)
    os.chdir("../../..")
    print("  Install complete.")

# ═══ LOAD MODEL ═══

def load_model():
    print("[3/4] Loading Hunyuan3D-2 model...")
    from hy3dgen.shapegen import Hunyuan3DDiTFlowMatchingPipeline
    from hy3dgen.texgen import Hunyuan3DPaintPipeline

    shape_pipe = Hunyuan3DDiTFlowMatchingPipeline.from_pretrained(
        "tencent/Hunyuan3D-2",
        subfolder="hunyuan3d-dit-v2-0",
        variant="fp16",
        device="cuda",
    )
    tex_pipe = Hunyuan3DPaintPipeline.from_pretrained(
        "tencent/Hunyuan3D-2",
        subfolder="hunyuan3d-paint-v2-0",
        device="cuda",
    )
    print("  Model loaded on GPU.")
    return shape_pipe, tex_pipe

# ═══ CONVERT ═══

def convert(shape_pipe, tex_pipe, char_key, image_b64):
    print(f"[4/4] Converting {char_key}...")

    # Decode image
    from PIL import Image
    import io
    img_data = base64.b64decode(image_b64)
    img = Image.open(io.BytesIO(img_data)).convert("RGB")

    # Generate shape
    print("  Generating mesh...")
    mesh = shape_pipe(img, num_inference_steps=50)
    print(f"  Mesh: {mesh.vertices.shape[0]} vertices, {mesh.faces.shape[0]} faces")

    # Generate texture
    if USE_TEXTURE:
        print("  Generating texture...")
        mesh = tex_pipe(mesh, img)
        print("  Texture applied.")

    # Export GLB
    import trimesh
    out_path = os.path.join(OUTPUT_DIR, f"{char_key}.glb")
    mesh.export(out_path)
    print(f"  Exported: {out_path}")
    return out_path

# ═══ FALLBACK: Stable Fast 3D (lighter, <1s) ═══

def convert_fast3d(char_key, image_b64):
    """Fallback if Hunyuan3D-2 OOMs or won't install."""
    subprocess.run([sys.executable, "-m", "pip", "install", "-q", "git+https://github.com/Stability-AI/stable-fast-3d.git"], check=True)

    from PIL import Image
    import io
    img_data = base64.b64decode(image_b64)
    img = Image.open(io.BytesIO(img_data)).convert("RGB")

    from sf3d.system import SF3D
    model = SF3D.from_pretrained("stabilityai/stable-fast-3d")
    mesh = model.run_image(img, remesh="quad")

    out_path = os.path.join(OUTPUT_DIR, f"{char_key}.glb")
    mesh.export(out_path)
    print(f"  Exported (fast3d): {out_path}")
    return out_path

# ═══ MAIN ═══

if __name__ == "__main__":
    print(f"=" * 50)
    print(f"KAGGLE GPU — Hunyuan3D-2 Image-to-3D")
    print(f"Character: {CHAR_KEY}")
    print(f"GPU: {subprocess.run(['nvidia-smi', '--query-gpu=name', '--format=csv,noheader'], capture_output=True, text=True).stdout.strip()}")
    print(f"=" * 50)

    if not IMAGE_B64:
        print("ERROR: AETHERION_IMAGE_B64 not set. Embed image before push.")
        sys.exit(1)

    t0 = time.time()

    try:
        install()
        shape_pipe, tex_pipe = load_model()
        out = convert(shape_pipe, tex_pipe, CHAR_KEY, IMAGE_B64)
    except Exception as e:
        print(f"Hunyuan3D-2 failed ({e}), falling back to Stable Fast 3D...")
        out = convert_fast3d(CHAR_KEY, IMAGE_B64)

    elapsed = time.time() - t0
    print(f"\nDone in {elapsed:.0f}s — {out}")
    print(f"Download: {out}")
