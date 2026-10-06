"""Opt-in local model fixtures, never downloads or installs into the user's profile."""
import importlib.util
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
from unittest.mock import patch

root = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("worker", root / "engine/local_engine.py")
worker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(worker)
binary = root / "src-tauri/binaries/local-engine-x86_64-pc-windows-msvc.exe"
if not binary.is_file():
    raise SystemExit("Build the Windows sidecar first")
with tempfile.TemporaryDirectory(prefix="engine-qa-", dir=root / ".tools") as temporary:
    data = Path(temporary)
    for model in worker.CATALOG:
        archive = root / ".tools/engine-models" / model["url"].rsplit("/", 1)[-1]
        worker.install(data, {"modelId":model["id"], "archive":str(archive)})
    cases = [("pt-BR", "en-US", "Bom dia!\nObrigado pela ajuda."), ("en-US", "pt-BR", "Good morning!\nThank you for your help."), ("pt-BR", "de-DE", "Bom dia! Obrigado pela ajuda."), ("de-DE", "pt-BR", "Guten Morgen! Vielen Dank für Ihre Hilfe.")]
    # Python API network denial verifies no hidden downloader in our inference path.
    with patch.object(socket.socket, "connect", side_effect=AssertionError("Network disabled for inference")):
        for source, target, text in cases:
            result = worker.handle(data, {"action":"translate", "source":source, "target":target, "text":text})
            assert result["text"].strip() and result["text"] != text
            assert "▁" not in result["text"]
            assert result["text"].count("\n") == text.count("\n")
    # Empty PATH: packaged Python/runtime, independent of globally installed Python.
    environment = {**os.environ, "PATH":"", "PYTHONHOME":"", "PYTHONPATH":""}
    for source, target, text in cases:
        started = time.monotonic()
        request = {"action":"translate", "source":source, "target":target, "text":text}
        process = subprocess.run([str(binary), "--data-dir", str(data)], input=(json.dumps(request)+"\n").encode(), capture_output=True, env=environment, timeout=60)
        result = json.loads(process.stdout)
        assert process.returncode == 0 and result["ok"], result.get("error")
        assert result["data"]["text"].strip() and result["data"]["text"] != text
        assert "▁" not in result["data"]["text"]
        assert result["data"]["text"].count("\n") == text.count("\n")
        assert not process.stderr, "Worker unexpectedly logged data"
        print(f"Packaged {source} -> {target}: {(time.monotonic()-started)*1000:.0f} ms; non-empty; newlines preserved; no stderr")
    # Cancellation closes the ownership pipe. Both bootloader and inference child
    # must exit; the managed worker does not remain as an orphaned engine.
    # Normal managed request must exit cleanly with the pipe still owned by Rust.
    process = subprocess.Popen([str(binary), "--data-dir", str(data), "--managed"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=environment)
    process.stdin.write((json.dumps({"action":"status"})+"\n").encode())
    process.stdin.flush()
    result = json.loads(process.stdout.readline())
    process.wait(timeout=15)
    process.stdin.close()
    assert result["ok"] and process.returncode == 0 and not process.stderr.read()
    print("Managed normal completion: clean exit, no stderr")
    process = subprocess.Popen([str(binary), "--data-dir", str(data), "--managed"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=environment)
    process.stdin.write((json.dumps({"action":"translate", "source":"pt-BR", "target":"de-DE", "text":"Bom dia. "*700})+"\n").encode())
    process.stdin.flush()
    time.sleep(0.2)
    process.stdin.close()
    process.wait(timeout=15)
    print("Managed cancellation: process exited after ownership pipe closed")
