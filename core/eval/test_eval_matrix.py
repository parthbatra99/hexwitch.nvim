import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).parent


class EvalMatrixTest(unittest.TestCase):
    def test_all_corpus_requests_have_preserved_annotations(self):
        corpus = {c["id"]: c for c in json.loads((ROOT / "corpus-v1.json").read_text())["cases"]}
        cases = json.loads((ROOT / "interpretation-matrix-v1.json").read_text())["cases"]
        originals = {c["id"]: c for c in cases if c["id"] in corpus}
        self.assertEqual(set(originals), set(corpus))
        for id, case in originals.items():
            self.assertEqual(case["request"], corpus[id]["request"])
            self.assertEqual(case["split"], corpus[id]["split"])
            self.assertEqual(case["category"], corpus[id]["category"])
        self.assertEqual(len(cases), 74)
        self.assertEqual(sum(c["previous"] is not None for c in cases if "previous" in c), 6)
        self.assertEqual(len({c["id"] for c in cases}), len(cases))

    def test_matrix_and_mutations_rebuild_identically(self):
        spec = importlib.util.spec_from_file_location("eval_matrix_builder", ROOT / "build-eval-matrix.py")
        builder = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(builder)
        with tempfile.TemporaryDirectory(prefix="hexwitch-eval-builder-") as directory:
            target = Path(directory)
            (target / "corpus-v1.json").write_bytes((ROOT / "corpus-v1.json").read_bytes())
            builder.ROOT = target
            builder.build()
            for file in ("interpretation-matrix-v1.json", "interpretation-matrix-replay.json", "states/dark.json", "states/light.json", "states/dark-extra-role.json"):
                self.assertEqual((target / file).read_bytes(), (ROOT / file).read_bytes(), file)
            spec = importlib.util.spec_from_file_location("eval_mutation_builder", ROOT / "build-eval-mutations.py")
            mutations = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(mutations)
            mutations.ROOT = target
            mutations.build()
            manifest = "mutations/manifest.json"
            self.assertEqual((target / manifest).read_bytes(), (ROOT / manifest).read_bytes(), manifest)

    def test_score_buckets_and_holdout_limitations_are_explicit(self):
        suite = json.loads((ROOT / "interpretation-matrix-v1.json").read_text())
        self.assertTrue(any("not a fresh independent holdout" in text for text in suite["limitations"]))
        self.assertEqual({c["expectation"] for c in suite["cases"]}, {"edit", "mixed", "clarification", "unsupported", "refusal"})
        run = json.loads((ROOT / "interpretation-matrix-replay.json").read_text())
        self.assertEqual(run["kind"], "canned_replay")
        self.assertEqual(run["provider"], "offline")
        self.assertEqual({r["case_id"] for r in run["responses"]}, {c["id"] for c in suite["cases"]})


if __name__ == "__main__":
    unittest.main()
