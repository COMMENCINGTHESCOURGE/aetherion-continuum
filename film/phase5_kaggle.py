"""
local : kaggle_cloud :: zero_dollars : zero_dollars
the bar between them IS the pipeline.
delete it, you have either local preview with no GPU export, or blind cloud export with no local control.
"""

import json, os, sys, base64, time, subprocess, urllib.request, urllib.error

BARS_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "tripo_bars.json")
SPRITE_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets", "sprites")
KERNEL_SRC = os.path.join(os.path.dirname(os.path.abspath(__file__)), "kaggle_hunyuan_kernel.py")

KAGGLE_API = "https://www.kaggle.com/api/v1"
KAGGLE_USER = "commencethescourge"

def load_bars():
    with open(BARS_PATH) as f:
        return json.load(f)

class KagglePipeline:
    def __init__(self, kaggle_username=None, kaggle_key=None):
        self.bars = load_bars()
        self.user = kaggle_username or KAGGLE_USER
        self.key = kaggle_key or self._load_kaggle_key()

    def _load_kaggle_key(self):
        kaggle_json = os.path.expanduser("~/.kaggle/kaggle.json")
        if os.path.exists(kaggle_json):
            with open(kaggle_json) as f:
                cfg = json.load(f)
            return cfg.get("key", "")
        return ""

    def _auth_header(self):
        import base64 as b64
        creds = b64.b64encode(f"{self.user}:{self.key}".encode()).decode()
        return {"Authorization": f"Basic {creds}", "Content-Type": "application/json"}

    # ═══ KAGGLE API ═══

    def _api(self, method, path, body=None):
        url = f"{KAGGLE_API}{path}"
        data = json.dumps(body).encode() if body else None
        req = urllib.request.Request(url, data=data, headers=self._auth_header(), method=method)
        try:
            with urllib.request.urlopen(req) as resp:
                return json.loads(resp.read())
        except urllib.error.HTTPError as e:
            err = e.read().decode()
            print(f"  Kaggle API error {e.code}: {err[:200]}")
            return None

    # ═══ PUSH KERNEL ═══

    def push_kernel(self, kernel_slug, char_key, image_path):
        """Push kernel with embedded image. kernel_slug must be an EXISTING kernel."""
        if not os.path.exists(image_path):
            print(f"  SKIP: image not found: {image_path}")
            return None

        # Encode image
        with open(image_path, "rb") as f:
            img_b64 = base64.b64encode(f.read()).decode()
        print(f"  Image: {os.path.basename(image_path)} — {len(img_b64)} chars base64")

        # Read kernel source and inject character + image
        with open(KERNEL_SRC) as f:
            kernel_code = f.read()

        # Inject env vars at top
        header = (
            f"import os\n"
            f"os.environ['AETHERION_CHAR_KEY'] = '{char_key}'\n"
            f"os.environ['AETHERION_IMAGE_B64'] = '{img_b64}'\n"
        )
        kernel_code = header + kernel_code

        # Push via REST API — update EXISTING kernel
        body = {
            "slug": kernel_slug,
            "newKernel": False,           # MUST be False — API regression blocks creation
            "kernelType": "script",        # auto-executes on push
            "acceleratorTypeGPU": "teslaP100",  # P100: 16GB HBM2, 9.3 TFLOPS
            "text": kernel_code,
            "title": f"Aetherion-Continuum Image-to-3D: {char_key}",
            "language": "python",
            "isPrivate": True,
        }

        print(f"  Pushing {char_key} → kaggle:{kernel_slug} (P100 GPU)...")
        result = self._api("POST", "/kernels/push", body)
        if result:
            print(f"  Push OK — kernel executing on Kaggle GPU")
            return result.get("slug") or kernel_slug
        return None

    # ═══ POLL + DOWNLOAD ═══

    def poll_status(self, kernel_slug, timeout_min=10):
        """Poll kernel status until complete. Returns output files list or None."""
        print(f"  Polling {kernel_slug}...")
        deadline = time.time() + timeout_min * 60
        while time.time() < deadline:
            result = self._api("GET", f"/kernels/{self.user}/{kernel_slug}/status")
            if not result:
                time.sleep(15)
                continue
            status = result.get("status", "unknown")
            print(f"    status={status}")
            if status == "complete":
                return result
            if status == "error" or status == "failed":
                print(f"  Kernel failed: {result}")
                return None
            time.sleep(30)
        print("  Timeout — kernel still running or queued")
        return None

    def list_outputs(self, kernel_slug):
        """List output files from a completed kernel."""
        return self._api("GET", f"/kernels/{self.user}/{kernel_slug}/output")

    def download_output(self, kernel_slug, output_file, dest_dir):
        """Download a file from kernel output."""
        url = f"{KAGGLE_API}/kernels/{self.user}/{kernel_slug}/output/{output_file}"
        req = urllib.request.Request(url, headers=self._auth_header())
        try:
            with urllib.request.urlopen(req) as resp:
                data = resp.read()
            dest = os.path.join(dest_dir, output_file)
            with open(dest, "wb") as f:
                f.write(data)
            return dest
        except urllib.error.HTTPError:
            return None

    # ═══ FULL PIPELINE ═══

    def convert(self, char_key, kernel_slug="sprite-delta-analyst", output_dir=None):
        """Convert one character: push → poll → download."""
        if not self.key:
            print("  SKIP: no Kaggle API key (~/.kaggle/kaggle.json)")
            return None

        clean_dir = os.path.join(SPRITE_DIR, "chain_refined")
        image_path = os.path.join(clean_dir, f"{char_key}_512.png")
        if not os.path.exists(image_path):
            image_path = os.path.join(clean_dir, f"{char_key}_full.png")
        if not os.path.exists(image_path):
            print(f"  SKIP: no clean image for {char_key}")
            return None

        if output_dir is None:
            output_dir = os.path.join(SPRITE_DIR, "kaggle_output")
        os.makedirs(output_dir, exist_ok=True)

        t0 = time.time()

        # Push
        slug = self.push_kernel(kernel_slug, char_key, image_path)
        if not slug:
            return None

        # Poll
        status = self.poll_status(slug, timeout_min=12)
        if not status:
            return None

        # Download GLB
        outputs = self.list_outputs(slug)
        glb_files = [f["name"] for f in (outputs or []) if f["name"].endswith(".glb")]

        result = None
        if glb_files:
            result = self.download_output(slug, glb_files[0], output_dir)

        elapsed = time.time() - t0
        print(f"  Kaggle pipeline complete in {elapsed:.0f}s")
        return result

    # ═══ BATCH ═══

    def batch_convert(self, char_keys, kernel_slug="sprite-delta-analyst", parallel=False):
        """Batch convert multiple characters. Kaggle allows 2 concurrent GPU sessions."""
        results = {}
        for ck in char_keys:
            out = self.convert(ck, kernel_slug)
            results[ck] = out
        return results

# ═══ CLI ═══

if __name__ == "__main__":
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--char", help="Character key to convert")
    ap.add_argument("--kernel", default="sprite-delta-analyst", help="Kaggle kernel slug to use")
    ap.add_argument("--batch", nargs="*", help="Multiple character keys")
    ap.add_argument("--list-outputs", action="store_true", help="List outputs from last kernel run")
    args = ap.parse_args()

    kp = KagglePipeline()

    if args.list_outputs:
        outputs = kp.list_outputs(args.kernel)
        if outputs:
            for f in outputs:
                print(f"  {f['name']:40s} {f.get('size', '?'):>10s}")
    elif args.batch:
        results = kp.batch_convert(args.batch, args.kernel)
        for ck, out in results.items():
            print(f"  {ck}: {out or 'FAILED'}")
    elif args.char:
        out = kp.convert(args.char, args.kernel)
        print(f"  result: {out or 'FAILED'}")
    else:
        ap.print_help()
