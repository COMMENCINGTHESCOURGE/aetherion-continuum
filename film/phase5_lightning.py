"""
lightning : kaggle : tripo  —  three cloud paths, one bar.
lightning costs credits but has a full SDK. kaggle is free but API is degraded.
tripo costs money but needs zero setup. the bar routes accordingly.
"""

import os, sys, json, base64, time, subprocess

BARS_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "tripo_bars.json")
SPRITE_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets", "sprites")
KERNEL_SRC = os.path.join(os.path.dirname(os.path.abspath(__file__)), "kaggle_hunyuan_kernel.py")

LIGHTNING_USER = "dashawnspacem"
LIGHTNING_TEAMSPACE = "erdos-straus-sieve-project"
LIGHTNING_API_KEY = "f98238d1-207c-46a0-a80d-9d87ad6fe7de"

def load_bars():
    with open(BARS_PATH) as f:
        return json.load(f)

class LightningPipeline:
    def __init__(self, api_key=None, user=None, teamspace=None):
        self.bars = load_bars()
        self.api_key = api_key or os.environ.get("LIGHTNING_API_KEY", LIGHTNING_API_KEY)
        self.user = user or LIGHTNING_USER
        self.teamspace = teamspace or LIGHTNING_TEAMSPACE
        self._sdk = None
        self._ts = None

    def _init_sdk(self):
        if self._sdk:
            return
        os.environ["LIGHTNING_API_KEY"] = self.api_key
        from lightning_sdk import Teamspace, Studio
        self._ts = Teamspace(name=self.teamspace, user=self.user)
        self._sdk = True

    # ═══ STUDIO MANAGEMENT ═══

    def _get_studio(self, name="controlled-copper-5tcd", switch_to_gpu=True):
        self._init_sdk()
        from lightning_sdk import Studio, Teamspace
        s = Studio(name=name, teamspace=self._ts)

        status = s.status
        print(f"  Studio: {name} | status={status}")

        if switch_to_gpu and status != "Running":
            try:
                s.switch_machine("t4")
                print("  Switched to T4 GPU")
            except Exception as e:
                print(f"  GPU switch failed (may already be T4): {e}")

        if status != "Running":
            print("  Starting studio (~30-60s)...")
            s.start()
            for _ in range(12):
                time.sleep(5)
                try:
                    if s.status == "Running":
                        break
                except:
                    pass
            print(f"  Studio: {s.status}")

        return s

    # ═══ CONVERT ═══

    def convert(self, char_key, studio_name="controlled-copper-5tcd", output_dir=None):
        if not self.api_key:
            print("  SKIP: no LIGHTNING_API_KEY")
            return None

        clean_dir = os.path.join(SPRITE_DIR, "chain_refined")
        image_path = os.path.join(clean_dir, f"{char_key}_512.png")
        if not os.path.exists(image_path):
            image_path = os.path.join(clean_dir, f"{char_key}_full.png")
        if not os.path.exists(image_path):
            print(f"  SKIP: no clean image for {char_key}")
            return None

        if output_dir is None:
            output_dir = os.path.join(SPRITE_DIR, "lightning_output")
        os.makedirs(output_dir, exist_ok=True)

        s = self._get_studio(studio_name)

        # Inject image as env var in kernel
        with open(image_path, "rb") as f:
            img_b64 = base64.b64encode(f.read()).decode()

        with open(KERNEL_SRC) as f:
            kernel_code = f.read()

        header = (
            f"import os\n"
            f"os.environ['AETHERION_CHAR_KEY'] = '{char_key}'\n"
            f"os.environ['AETHERION_IMAGE_B64'] = '{img_b64}'\n"
        )
        kernel_code = header + kernel_code

        # Write modified kernel to temp file
        tmp_kernel = os.path.join(output_dir, f"_lightning_{char_key}.py")
        with open(tmp_kernel, "w") as f:
            f.write(kernel_code)

        # Upload and run
        print(f"  Uploading kernel ({len(kernel_code)} bytes)...")
        s.upload_file(tmp_kernel, f"hunyuan_{char_key}.py")

        print(f"  Running Hunyuan3D-2 on Lightning T4 GPU...")
        t0 = time.time()
        output = s.run(f"python hunyuan_{char_key}.py")

        elapsed = time.time() - t0
        print(f"  Done in {elapsed:.0f}s")

        # Download GLB
        try:
            s.download_file(f"{char_key}.glb", os.path.join(output_dir, f"{char_key}.glb"))
            print(f"  Downloaded: {output_dir}/{char_key}.glb")
            return os.path.join(output_dir, f"{char_key}.glb")
        except Exception as e:
            print(f"  Download failed: {e}")
            print(f"  Output was:\n{output[:500] if output else '(empty)'}")
            return None

    # ═══ BATCH ═══

    def batch_convert(self, char_keys, studio_name="controlled-copper-5tcd"):
        results = {}
        for ck in char_keys:
            out = self.convert(ck, studio_name)
            results[ck] = out
        return results

# ═══ CLI ═══

if __name__ == "__main__":
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--char", help="Character key to convert")
    ap.add_argument("--studio", default="controlled-copper-5tcd", help="Lightning studio name")
    ap.add_argument("--batch", nargs="*", help="Multiple character keys")
    ap.add_argument("--status", action="store_true", help="Check studio status")
    args = ap.parse_args()

    lp = LightningPipeline()

    if args.status:
        s = lp._get_studio(args.studio, switch_to_gpu=False)
    elif args.batch:
        results = lp.batch_convert(args.batch, args.studio)
        for ck, out in results.items():
            print(f"  {ck}: {out or 'FAILED'}")
    elif args.char:
        out = lp.convert(args.char, args.studio)
        print(f"  result: {out or 'FAILED'}")
    else:
        ap.print_help()
