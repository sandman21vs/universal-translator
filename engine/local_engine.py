"""Private stdio worker. No HTTP listener, network client, logs or auto-downloads."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import sys
import tempfile
import threading
import zipfile

CATALOG = json.loads((Path(__file__).parent / "catalog.json").read_text(encoding="utf-8"))


def model_path(root, model):
    return root / model["id"]


def installed(root, model):
    path = model_path(root, model)
    try:
        receipt = json.loads((path / "receipt.json").read_text())
        return receipt["sha256"] == model["sha256"] and (path / "model/model.bin").is_file() and (path / "sentencepiece.model").is_file()
    except (OSError, ValueError, KeyError):
        return False


def install(root, request):
    model = next((m for m in CATALOG if m["id"] == request["modelId"]), None)
    if model is None:
        raise ValueError("Modelo fora do catálogo verificado.")
    archive = Path(request["archive"])
    with archive.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    if archive.stat().st_size != model["bytes"] or digest != model["sha256"]:
        raise ValueError("Checksum ou tamanho do modelo inválido.")
    root.mkdir(parents=True, exist_ok=True)
    destination = model_path(root, model)
    if installed(root, model):
        return {"installed": True}
    with tempfile.TemporaryDirectory(prefix="install-", dir=root) as temporary:
        stage = Path(temporary)
        with zipfile.ZipFile(archive) as package:
            entries = package.infolist()
            if len(entries) > 300 or sum(e.file_size for e in entries) > 600_000_000:
                raise ValueError("Arquivo de modelo excede os limites.")
            for entry in entries:
                parts = PurePosixPath(entry.filename).parts
                if not parts or any(p in ("..", ".") or ":" in p or "\\" in p for p in parts) or entry.filename.startswith("/") or (entry.external_attr >> 16) & 0o170000 == 0o120000:
                    raise ValueError("Arquivo de modelo contém caminho inválido.")
                # Only inference data. No Stanza/Python assets are executed or installed.
                relative = parts[1:]
                if not relative or entry.is_dir():
                    continue
                if relative[0] != "model" and relative[0] not in ("sentencepiece.model", "metadata.json", "LICENSE", "README.md"):
                    continue
                target = stage.joinpath(*relative)
                target.parent.mkdir(parents=True, exist_ok=True)
                with package.open(entry) as source, target.open("wb") as output:
                    shutil.copyfileobj(source, output)
        metadata = json.loads((stage / "metadata.json").read_text())
        if metadata.get("from_code") != model["source"] or metadata.get("to_code") != model["target"] or metadata.get("package_version") != model["version"]:
            raise ValueError("Metadados do modelo incompatíveis.")
        if not (stage / "model/model.bin").is_file() or not (stage / "sentencepiece.model").is_file():
            raise ValueError("Modelo sem dados de inferência.")
        (stage / "receipt.json").write_text(json.dumps({"sha256": digest}), encoding="utf-8")
        (stage / "manifest.json").write_text(json.dumps(model, ensure_ascii=False, indent=2), encoding="utf-8")
        (stage / "THIRD_PARTY.md").write_text((Path(__file__).parent / "THIRD_PARTY.md").read_text(encoding="utf-8"), encoding="utf-8")
        if destination.exists():
            raise ValueError("Modelo existente inválido. Remova sua pasta antes de reinstalar.")
        stage.rename(destination)
    return {"installed": True}


def translate(root, request):
    source, target, text = request["source"].split("-")[0], request["target"].split("-")[0], request["text"]
    if source == "auto":
        raise ValueError("No modo local integrado, escolha o idioma de origem nas configurações.")
    if not isinstance(text, str) or not text.strip() or len(text) > 8000:
        raise ValueError("Texto vazio ou longo demais.")
    available = [m for m in CATALOG if installed(root, m)]
    direct = next((m for m in available if m["source"] == source and m["target"] == target), None)
    if source == target:
        return {"text": text}
    if direct:
        route = [direct]
    else:
        first = next((m for m in available if m["source"] == source and m["target"] == "en"), None)
        second = next((m for m in available if m["source"] == "en" and m["target"] == target), None)
        if not first or not second:
            raise ValueError("Par de idiomas não instalado. Instale os modelos em Configurações → Local integrado.")
        route = [first, second]
    # Lazy imports: listing/installing models never loads the inference runtime.
    import ctranslate2
    import sentencepiece
    for model in route:
        path = model_path(root, model)
        tokenizer = sentencepiece.SentencePieceProcessor(model_file=str(path / "sentencepiece.model"))
        translator = ctranslate2.Translator(str(path / "model"), device="cpu", compute_type="int8", inter_threads=1, intra_threads=min(4, os.cpu_count() or 1))
        paragraphs = re.split(r"(\r\n|\n|\r)", text)
        for index in range(0, len(paragraphs), 2):
            paragraph = paragraphs[index]
            if not paragraph.strip():
                continue
            sentences = re.split(r"(?<=[.!?])\s+", paragraph.strip())
            tokens = [tokenizer.encode(s, out_type=str) for s in sentences]
            if any(len(t) > 1024 for t in tokens):
                raise ValueError("Uma frase excede o limite do motor local. Divida em frases menores.")
            batches = translator.translate_batch(tokens, beam_size=4, replace_unknowns=True, max_input_length=1024, max_decoding_length=2048)
            if any(len(b.hypotheses[0]) >= 2048 for b in batches):
                raise ValueError("Tradução local incompleta. Divida o texto em partes menores.")
            leading = paragraph[:len(paragraph) - len(paragraph.lstrip())]
            trailing = paragraph[len(paragraph.rstrip()):]
            # Some OPUS vocabularies emit literal SentencePiece space markers.
            # Normalize those markers only; underscores in identifiers stay intact.
            paragraphs[index] = leading + " ".join(tokenizer.decode(b.hypotheses[0]).replace("▁", " ").lstrip(" ") for b in batches) + trailing
        text = "".join(paragraphs)
        del translator
    return {"text": text}


def handle(root, request):
    if request.get("action") == "status":
        return {"models": [{**m, "installed": installed(root, m)} for m in CATALOG]}
    if request.get("action") == "install":
        return install(root, request)
    if request.get("action") == "translate":
        return translate(root, request)
    raise ValueError("Operação local inválida.")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--data-dir", required=True)
    parser.add_argument("--managed", action="store_true")
    args = parser.parse_args()
    try:
        line = sys.stdin.buffer.readline(65537)
        if len(line) > 65536:
            raise ValueError("Requisição local excede o limite.")
        if args.managed:
            # Closing the Rust pipe also stops the PyInstaller inference child.
            def parent_lifetime():
                # Raw descriptor: a daemon must not hold Python's buffered-I/O
                # lock during interpreter finalization after a normal reply.
                os.read(sys.stdin.fileno(), 1)
                os._exit(0)
            threading.Thread(target=parent_lifetime, daemon=True).start()
        result = {"ok": True, "data": handle(Path(args.data_dir), json.loads(line))}
    except ValueError as error:
        # Only our predefined messages, never request text / paths from parser errors.
        safe = str(error) if str(error).startswith(("Modelo ", "Checksum ", "Arquivo de modelo ", "Metadados ", "No modo ", "Texto vazio ", "Par de ", "Uma frase ", "Tradução local ", "Operação local ", "Requisição local ")) else "Dados do motor local inválidos."
        result = {"ok": False, "error": safe}
    except Exception:
        result = {"ok": False, "error": "Falha no motor local. Verifique a instalação dos modelos."}
    sys.stdout.buffer.write((json.dumps(result, ensure_ascii=True) + "\n").encode("ascii"))
    sys.stdout.buffer.flush()


if __name__ == "__main__":
    main()
