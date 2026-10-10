#!/usr/bin/env python3
"""Build deliberately hand-authored contract replay, never a model benchmark."""
import json
from pathlib import Path

ROOT = Path(__file__).parent


def action(op, roles, property_name, **fields):
    record = dict(op=op, roles=roles, property=property_name, include_semantic=False,
                  scope=None, color=None, number=None, boolean=None, underline=None,
                  state=None, backgrounds=None, min=None, direction=None, persistent=None)
    record.update(fields)
    return record


def ready(actions):
    return dict(version=1, decision="ready", notes="Hand-authored property contract replay.", questions=[], actions=actions)


def decision(kind, note, questions=None):
    return dict(version=1, decision=kind, notes=note, questions=questions or [], actions=[])


responses = {
    "i05": ready([action("lock", ["surface.base"], "bg", scope="persistent"),
                  action("contrast", ["syntax.comment"], "fg", backgrounds=[dict(surface="surface.base", overlays=[])], min=4.5, direction="both", persistent=False)]),
    "i07": json.loads((ROOT / "llm/property-opaque-background.json").read_text()),
    "i13": json.loads((ROOT / "llm/clarify-blue.json").read_text()),
    "i16": ready([action("lightness", ["surface.base"], "bg", number=-0.02)]),
    "i17": ready([action("lightness", ["surface.base"], "bg", number=-0.08)]),
    "i19": decision("needs_clarification", "The comment protection conflicts with the requested change.", ["May comment foreground protection be removed, or should comments remain unchanged?"]),
    "i21": decision("unsupported", "Font size is an editor setting, not a theme property."),
    "i29": ready([action("set_bool", ["*"], "italic", boolean=False, include_semantic=True)]),
    "i32": ready([action("set_color", ["syntax.string"], "fg", color="#4f9077")]),
    "i37": decision("needs_clarification", "Cold may refer to neutrals or syntax accents.", ["Do you mean the background tint or particular syntax accents?"]),
    "i46": ready([action("unlock", ["interaction.selection"], "bg_opacity")]),
    "i50": ready([action("set_bool", ["syntax.comment"], "bold", boolean=False),
                  action("set_bool", ["syntax.comment"], "italic", boolean=False),
                  action("state", ["syntax.comment"], "underline", state="clear"),
                  action("set_bool", ["syntax.comment"], "strikethrough", boolean=False)]),
}
run = dict(version=1, label="Hand-authored v1 contract replay", provider="offline", model="none",
           prompt_version="property-intent-1", kind="canned_replay",
           responses=[dict(case_id=key, response=value) for key, value in responses.items()])
(ROOT / "interpretation-contract-replay.json").write_text(json.dumps(run, indent=2) + "\n")
