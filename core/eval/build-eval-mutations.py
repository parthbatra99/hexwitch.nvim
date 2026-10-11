#!/usr/bin/env python3
"""Known-bad perturbations test whether the evaluator detects regressions.

They are not alternate model runs and must never appear as benchmark evidence.
Each mutation copies the entire authored replay and names the checks it should
fail. Source annotations are never changed to accommodate these outputs.
"""
import copy
import json
from pathlib import Path

ROOT = Path(__file__).parent


def build():
    replay = json.loads((ROOT / "interpretation-matrix-replay.json").read_text())
    directory = ROOT / "mutations"
    directory.mkdir(exist_ok=True)
    entries = []

    def mutate(name, case_id, transform, failure_kind, *, pair=None):
        run = copy.deepcopy(replay)
        run["label"] = "Known-bad evaluator sensitivity probe: " + name
        response = next(r["response"] for r in run["responses"] if r["case_id"] == case_id)
        transform(response)
        filename = name + ".json"
        (directory / filename).write_text(json.dumps(run, indent=2) + "\n")
        entry = dict(file=filename, case_id=case_id, expected_check=failure_kind)
        if pair:
            entry["expected_pair"] = pair
        entries.append(entry)

    mutate("wrong-role", "i01", lambda r: r["actions"][0].update(roles=["surface.panel"]), "unrelated_changes")
    mutate("implicit-whole-style-unlock", "i46", lambda r: r["actions"][0].update(property="bg"), "protection")
    mutate("unrelated-palette-change", "i29", lambda r: r["actions"].append(dict(r["actions"][0], op="chroma", roles=["syntax.string"], property="fg", boolean=None, number=-0.02)), "unrelated_changes")
    mutate("missed-semantic-italics", "i29", lambda r: r["actions"][0].update(include_semantic=False), "property_correctness")
    mutate("unnecessary-clarification", "i02", lambda r: r.update(decision="needs_clarification", questions=["Do you really want lighter comments?"], actions=[]), "strength")
    mutate("invented-anchor", "i40", lambda r: r["actions"][0].update(op="set_color", number=None, color="#0000ff"), "interpretation_validation")
    mutate("overbroad-lock", "i06", lambda r: r["actions"][0].update(roles=["*"]), "protection")
    # The individual strength intervals still pass, but the required pair gap
    # does not. This confirms pair failure changes the CLI's top-level outcome.
    mutate("pair-only-strength-gap", "i16", lambda r: r["actions"][0].update(number=-0.039), "pair_only", pair="slightly-versus-much")
    mutate("weak-much-change", "i17", lambda r: r["actions"][0].update(number=-0.02), "strength", pair="slightly-versus-much")
    mutate("lost-sequential-lock", "seq-refused", lambda r: r["actions"].insert(0, dict(r["actions"][0], op="unlock", number=None)), "protection")
    mutate("base-only-selection-floor", "i23", lambda r: r["actions"][0].update(backgrounds=[dict(surface="surface.base", overlays=[])]), "readability")
    mutate("omitted-persistent-floor", "seq-readable", lambda r: r["actions"][0].update(persistent=False), "readability")
    mutate("cleared-unrelated-decoration", "i49", lambda r: r["actions"].append(dict(r["actions"][0], op="set_bool", property="strikethrough", state=None, boolean=False)), "preservation")
    (directory / "manifest.json").write_text(json.dumps(dict(version=1, purpose="Known-bad evaluator sensitivity, not model results", mutations=entries), indent=2) + "\n")
    print(f"Authored {len(entries)} known-bad sensitivity probes")


if __name__ == "__main__":
    build()
