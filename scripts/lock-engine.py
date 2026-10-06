"""Developer-only refresh of hashes for the exact pinned PyPI distributions."""
import json
from pathlib import Path
import urllib.request

root = Path(__file__).resolve().parents[1]
lines = ["# Generated from exact PyPI versions; all published wheel/sdist hashes."]
for requirement in (root / "engine/requirements.txt").read_text().splitlines():
    if not requirement or requirement.startswith("#"):
        continue
    pin = requirement.split(";")[0].strip()
    name, version = pin.split("==")
    with urllib.request.urlopen(f"https://pypi.org/pypi/{name}/{version}/json", timeout=30) as response:
        data = json.load(response)
    hashes = sorted({entry["digests"]["sha256"] for entry in data["urls"]})
    if not hashes:
        raise SystemExit(f"Missing distribution hashes: {pin}")
    lines.append(requirement + " \\")
    for index, digest in enumerate(hashes):
        lines.append("    --hash=sha256:" + digest + (" \\" if index < len(hashes)-1 else ""))
(root / "engine/requirements.lock").write_text("\n".join(lines)+"\n", encoding="utf-8")
print("Wrote engine/requirements.lock with verified PyPI distribution hashes")
