import json
import unittest
from pathlib import Path

ROOT = Path(__file__).parent


class ContractReplayTest(unittest.TestCase):
    def test_cases_match_annotated_corpus_and_split(self):
        corpus = json.loads((ROOT / "corpus-v1.json").read_text())
        suite = json.loads((ROOT / "interpretation-suite-v1.json").read_text())
        annotations = {case["id"]: case for case in corpus["cases"]}
        for case in suite["cases"]:
            self.assertIn(case["id"], annotations)
            source = annotations[case["id"]]
            self.assertEqual(case["request"], source["request"])
            self.assertEqual(case["split"], source["split"])
            self.assertEqual(case["category"], source["category"])
        self.assertEqual(sum(case["split"] == "held_out" for case in suite["cases"]), 4)

    def test_canned_run_is_labeled_and_complete(self):
        run = json.loads((ROOT / "interpretation-contract-replay.json").read_text())
        suite = json.loads((ROOT / "interpretation-suite-v1.json").read_text())
        self.assertEqual(run["kind"], "canned_replay")
        self.assertEqual(run["provider"], "offline")
        self.assertEqual({record["case_id"] for record in run["responses"]}, {case["id"] for case in suite["cases"]})


if __name__ == "__main__":
    unittest.main()
