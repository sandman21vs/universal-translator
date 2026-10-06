"""Explicit developer action: prepare pinned engine dependencies and binary."""
from pathlib import Path
import subprocess
import sys
import venv

root = Path(__file__).resolve().parents[1]
if sys.version_info[:2] != (3, 12):
    raise SystemExit("Use Python 3.12 to build the bundled engine")
environment = root / ".tools/local-engine"
venv.EnvBuilder(with_pip=True).create(environment)
python = environment / ("Scripts/python.exe" if sys.platform == "win32" else "bin/python")
subprocess.run([str(python), "-m", "pip", "install", "--require-hashes", "-r", str(root / "engine/requirements.lock")], check=True)
subprocess.run([str(python), str(root / "scripts/build-engine.py")], cwd=root, check=True)
