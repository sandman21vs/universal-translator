import hashlib
import importlib.util
import json
from pathlib import Path
import socket
import tempfile
import unittest
from unittest.mock import patch
import zipfile

spec = importlib.util.spec_from_file_location("worker", Path(__file__).with_name("local_engine.py"))
worker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(worker)


class InstallationTests(unittest.TestCase):
    def fixture(self, root, unsafe=False):
        archive = root / "test.argosmodel"
        with zipfile.ZipFile(archive, "w") as package:
            package.writestr("package/metadata.json", json.dumps({"from_code": "pt", "to_code": "en", "package_version": "1.0"}))
            package.writestr("package/model/model.bin", b"fixture-not-an-inference-model")
            package.writestr("package/sentencepiece.model", b"fixture")
            if unsafe:
                package.writestr("package/../../escaped", b"must-not-be-written")
        model = {"id": "pt-en-1.0", "source": "pt", "target": "en", "version": "1.0", "bytes": archive.stat().st_size, "sha256": hashlib.sha256(archive.read_bytes()).hexdigest()}
        return archive, model

    def test_install_receipt_and_corrupt_download(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive, model = self.fixture(root)
            with patch.object(worker, "CATALOG", [model]):
                request = {"modelId": model["id"], "archive": str(archive)}
                worker.install(root / "models", request)
                self.assertTrue(worker.installed(root / "models", model))
                archive.write_bytes(b"corrupt")
                with self.assertRaisesRegex(ValueError, "Checksum"):
                    worker.install(root / "other", request)
                self.assertFalse((root / "other").exists())

    def test_verified_archive_still_rejects_path_traversal(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive, model = self.fixture(root, unsafe=True)
            with patch.object(worker, "CATALOG", [model]):
                with self.assertRaisesRegex(ValueError, "caminho inválido"):
                    worker.install(root / "models", {"modelId": model["id"], "archive": str(archive)})
                self.assertFalse((root / "escaped").exists())
                self.assertFalse((root / "models" / model["id"]).exists())

    def test_auto_source_and_missing_pair_never_attempt_network(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(socket.socket, "connect", side_effect=AssertionError("network forbidden")):
            with self.assertRaisesRegex(ValueError, "idioma de origem"):
                worker.translate(Path(temporary), {"source": "auto", "target": "en-US", "text": "Olá"})
            with self.assertRaisesRegex(ValueError, "não instalado"):
                worker.translate(Path(temporary), {"source": "pt-BR", "target": "en-US", "text": "Olá"})


if __name__ == "__main__":
    unittest.main()
