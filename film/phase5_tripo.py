"""
local_preview : tripo_production
the bar between them IS the pipeline.
delete it, you have either infinite preview with no export, or blind export with no preview.
"""

import json, os, sys, base64, time, urllib.request, urllib.error

BARS_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "tripo_bars.json")
SPRITE_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets", "sprites")

def load_bars():
    with open(BARS_PATH) as f:
        return json.load(f)

class TripoPipeline:
    def __init__(self, api_key=None):
        self.bars = load_bars()
        self.api_key = api_key or os.environ.get("TRIPO_API_KEY", "")
        self.endpoint = self.bars["phase5"]["endpoint"]

    # ═══ DECISION ENGINE ═══

    def route(self, intent):
        """intent: 'preview' | 'iterate' | 'final_mesh' | 'textured' | 'rigged' | 'variant' | 'sprite_sheet' | 'game_ready'
        Returns: 'local' | 'tripo' | 'hybrid'"""
        routing = self.bars["route_logic"]
        if intent in routing:
            return routing[intent]

        db = self.bars["decision_bars"]
        if intent == "final":
            tripo_score = 0
            local_score = 0
            for dim, bar in db.items():
                if bar["route"] == "tripo":
                    tripo_score += 1
                elif bar["route"] == "local":
                    local_score += 1
            return "tripo" if tripo_score > local_score else "hybrid"
        return "local"

    # ═══ TRI3D API ═══

    def _api(self, method, path, body=None):
        url = f"https://api.tripo3d.ai{path}"
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
            "Accept": "application/json",
        }
        data = json.dumps(body).encode() if body else None
        req = urllib.request.Request(url, data=data, headers=headers, method=method)
        try:
            with urllib.request.urlopen(req) as resp:
                return json.loads(resp.read())
        except urllib.error.HTTPError as e:
            return {"code": e.code, "error": e.read().decode()}

    def submit(self, image_path, model_version=None, face_limit=None, texture=None, pbr=None, fmt=None):
        """Submit an image for 3D conversion. Returns task_id."""
        p5 = self.bars["phase5"]
        with open(image_path, "rb") as f:
            b64 = base64.b64encode(f.read()).decode()
        ext = image_path.split(".")[-1].lower()
        data_uri = f"data:image/{ext};base64,{b64}"

        body = {
            "type": "image_to_model",
            "file": data_uri,
            "model_version": model_version or p5["default_model"],
            "face_limit": face_limit or p5["default_faces"],
            "texture": texture if texture is not None else p5["default_texture"],
            "pbr": pbr if pbr is not None else p5["default_pbr"],
            "format": fmt or p5["default_format"],
        }
        result = self._api("POST", "/v1/task", body)
        if result.get("code") == 0:
            return result["data"]["task_id"]
        return None

    def poll(self, task_id, timeout_s=300):
        """Poll until success or failure. Returns output dict or None."""
        interval = self.bars["phase5"]["poll_interval_s"]
        elapsed = 0.0
        while elapsed < timeout_s:
            result = self._api("GET", f"/v1/task/{task_id}")
            if result.get("code") != 0:
                return None
            data = result["data"]
            if data["status"] == "success":
                return data.get("output", {})
            if data["status"] == "failed":
                return None
            time.sleep(interval)
            elapsed += interval
        return None

    def download(self, url, output_path):
        """Download model file from signed URL."""
        req = urllib.request.Request(url)
        with urllib.request.urlopen(req) as resp:
            with open(output_path, "wb") as f:
                f.write(resp.read())
        return output_path

    # ═══ FULL PIPELINE ═══

    def convert(self, char_key, intent="game_ready", output_dir=None):
        """Full conversion: route → execute → return path."""
        route = self.route(intent)
        print(f"  {char_key}: intent={intent} → route={route}")

        if route == "local":
            return self._local_path(char_key)
        elif route == "tripo":
            return self._tripo_path(char_key, output_dir)
        else:
            local = self._local_path(char_key)
            tripo = self._tripo_path(char_key, output_dir)
            return {"local": local, "tripo": tripo}

    def _local_path(self, char_key):
        clean_dir = os.path.join(SPRITE_DIR, "chain_refined")
        clean_file = os.path.join(clean_dir, f"{char_key}_512.png")
        if os.path.exists(clean_file):
            return clean_file
        clean_file = os.path.join(clean_dir, f"{char_key}_full.png")
        return clean_file if os.path.exists(clean_file) else None

    def _tripo_path(self, char_key, output_dir=None):
        if not self.api_key:
            print(f"  SKIP tripo: no TRIPO_API_KEY set")
            return None

        clean_file = self._local_path(char_key)
        if not clean_file:
            print(f"  SKIP tripo: no clean PNG for {char_key}")
            return None

        if output_dir is None:
            output_dir = os.path.join(SPRITE_DIR, "tripo_output")
        os.makedirs(output_dir, exist_ok=True)

        print(f"  submit: {clean_file}")
        task_id = self.submit(clean_file)
        if not task_id:
            print(f"  ERROR: submission failed")
            return None

        print(f"  task_id: {task_id} — polling...")
        output = self.poll(task_id)
        if not output or "model_url" not in output:
            print(f"  ERROR: generation failed or timed out")
            return None

        out_path = os.path.join(output_dir, f"{char_key}.glb")
        self.download(output["model_url"], out_path)
        print(f"  done: {out_path}")
        return out_path

    # ═══ BATCH ═══

    def batch_convert(self, char_keys, intent="game_ready"):
        results = {}
        cost_bars = self.bars["cost_bars"]
        model_ver = self.bars["phase5"]["default_model"]
        credit_cost = cost_bars[model_ver]["credits_per_model"]
        total = 0

        for ck in char_keys:
            out = self.convert(ck, intent)
            results[ck] = out
            if out:
                total += credit_cost

        results["_total_credits"] = total
        results["_tier"] = self._estimate_tier(total)
        return results

    def _estimate_tier(self, credits):
        for tier, bar in self.bars["rate_bars"].items():
            if credits <= bar["credits_per_month"]:
                return tier
        return "enterprise"

# ═══ CLI ═══

if __name__ == "__main__":
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--char", help="Character key to convert")
    ap.add_argument("--intent", default="game_ready",
        choices=["preview","iterate","final_mesh","textured","rigged","variant","sprite_sheet","game_ready"])
    ap.add_argument("--api-key", help="Tripo3D API key (or set TRIPO_API_KEY env var)")
    ap.add_argument("--batch", nargs="*", help="Multiple character keys for batch conversion")
    ap.add_argument("--list-routes", action="store_true", help="Show routing table")
    args = ap.parse_args()

    tp = TripoPipeline(args.api_key)

    if args.list_routes:
        print("intent               → route")
        print("─" * 40)
        for intent, route in tp.bars["route_logic"].items():
            print(f"{intent:20s} → {route}")
        print()
        print("decision bars:")
        for dim, bar in tp.bars["decision_bars"].items():
            lv = bar.get("local") or bar.get("local_max")
            tv = bar.get("tripo") or bar.get("tripo_min") or bar.get("tripo_max") or bar.get("tripo_per")
            print(f"  {dim:15s}: local={str(lv):10s} tripo={str(tv):10s} → {bar['route']}")
        sys.exit(0)

    if args.batch:
        results = tp.batch_convert(args.batch, args.intent)
        for ck, out in results.items():
            if not ck.startswith("_"):
                print(f"  {ck}: {out}")
        print(f"\n  total credits: {results['_total_credits']}")
        print(f"  sufficient tier: {results['_tier']}")
    elif args.char:
        out = tp.convert(args.char, args.intent)
        print(f"  result: {out}")
    else:
        ap.print_help()
