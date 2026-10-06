"""Run with Python 3.12 from an isolated venv containing engine/requirements.txt."""
import importlib.metadata
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
machine = platform.machine().lower()
targets = {("Windows", "amd64"): "x86_64-pc-windows-msvc", ("Linux", "x86_64"): "x86_64-unknown-linux-gnu", ("Darwin", "x86_64"): "x86_64-apple-darwin", ("Darwin", "arm64"): "aarch64-apple-darwin"}
target = targets.get((platform.system(), machine))
if not target or sys.version_info[:2] != (3, 12):
    raise SystemExit("Engine build requires Python 3.12 and a supported native architecture")
work = root / ".tools/engine-build" / target
notices = work / "licenses"
notices.mkdir(parents=True, exist_ok=True)
inventory = []
for distribution in importlib.metadata.distributions():
    name = distribution.metadata["Name"]
    inventory.append({"name": name, "version": distribution.version})
    for file in distribution.files or []:
        if "license" in str(file).lower() or "copying" in str(file).lower():
            source = Path(distribution.locate_file(file))
            if source.is_file():
                destination = notices / name / str(file).replace("/", "_").replace("\\", "_")
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(source, destination)
python_license = Path(sys.base_prefix) / "LICENSE.txt"
if python_license.is_file():
    shutil.copy2(python_license, notices / "Python-LICENSE.txt")
(notices / "inventory.json").write_text(json.dumps(inventory, indent=2), encoding="utf-8")
shutil.copy2(root / "engine/THIRD_PARTY.md", notices / "THIRD_PARTY.md")
shutil.copytree(root / "engine/licenses", notices / "upstream", dirs_exist_ok=True)
subprocess.run([sys.executable, "-m", "PyInstaller", "--noconfirm", "--clean", "--onefile", "--console", "--name", "local-engine", "--distpath", str(work / "dist"), "--workpath", str(work / "work"), "--specpath", str(work), "--additional-hooks-dir", str(root / "engine/hooks"), "--collect-all", "sentencepiece", "--add-data", str(root / "engine/catalog.json") + ":.", "--add-data", str(root / "engine/THIRD_PARTY.md") + ":.", "--add-data", str(notices) + ":licenses", str(root / "engine/local_engine.py")], cwd=root, check=True)
binary = work / "dist" / ("local-engine.exe" if platform.system() == "Windows" else "local-engine")
destination = root / "src-tauri/binaries" / ("local-engine-" + target + (".exe" if platform.system() == "Windows" else ""))
destination.parent.mkdir(parents=True, exist_ok=True)
shutil.copy2(binary, destination)
print("Built", destination)
