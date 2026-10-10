import copy
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("verify_fixtures", Path(__file__).with_name("verify-fixtures.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class FixtureMetadataTest(unittest.TestCase):
    def test_checked_in_manifest(self):
        self.assertEqual(module.verify(Path(__file__).parent / "import/manifest.json"), 5)

    def test_invalid_metadata_and_escaping_paths_fail(self):
        original = json.loads((Path(__file__).parent / "import/manifest.json").read_text())
        original["references"] = [original["references"][0]]
        with tempfile.TemporaryDirectory(prefix="hexwitch-metadata-") as directory:
            path = Path(directory) / "manifest.json"
            filename = original["references"][0]["file"]
            fixture = Path(directory) / filename
            fixture.write_bytes((Path(__file__).parent / "import" / filename).read_bytes())
            path.write_text(json.dumps(original))
            self.assertEqual(module.verify(path), 1)
            for mutation in [lambda f: f.pop("purpose"), lambda f: f.update(file="../escape.json"), lambda f: f.update(unexpected=True), lambda f: f.update(sha256="not-a-hash"), lambda f: f.update(license_file="../notice")]:
                data = copy.deepcopy(original)
                mutation(data["references"][0])
                path.write_text(json.dumps(data))
                with self.assertRaises(ValueError):
                    module.verify(path)
            path.write_text(json.dumps(original))
            fixture.write_text("changed bytes")
            with self.assertRaisesRegex(ValueError, "bytes changed"):
                module.verify(path)


if __name__ == "__main__":
    unittest.main()
