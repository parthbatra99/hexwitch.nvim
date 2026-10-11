#!/usr/bin/env python3
import copy
import json
from pathlib import Path

ROOT = Path(__file__).parent
DARK = "states/dark.json"
LIGHT = "states/light.json"
BG = [{"surface": "surface.base", "overlays": []}]
SELECTED = [{"surface": "surface.base", "overlays": ["interaction.selection"]}]


def target(role, prop="fg", semantic=False):
    return dict(roles=[role] if isinstance(role, str) else role, property=prop, include_semantic=semantic)


def check(kind, role=None, prop="fg", semantic=False, **fields):
    result = dict(check=kind, **fields)
    if role is not None:
        result["target"] = target(role, prop, semantic)
    return result


def only(*targets):
    return dict(check="only_changed", allowed=list(targets))


def locks(*targets):
    return dict(check="only_lock_changes", allowed=list(targets))


def action(op, role, prop="fg", semantic=False, **fields):
    result = dict(op=op, roles=[role] if isinstance(role, str) else role, property=prop,
                  include_semantic=semantic, scope=None, color=None, number=None,
                  boolean=None, underline=None, state=None, backgrounds=None,
                  min=None, direction=None, persistent=None)
    result.update(fields)
    return result


def ready(*actions):
    return dict(version=1, decision="ready", notes="Authored contract response; not provider output.", questions=[], actions=list(actions))


def clarify(question):
    return dict(version=1, decision="needs_clarification", notes="The reference or request does not identify one safe interpretation.", questions=[question], actions=[])


def unsupported(note):
    return dict(version=1, decision="unsupported", notes=note, questions=[], actions=[])


def outcome(*values):
    return dict(check="outcome", allowed=list(values))


def clarification_checks():
    return [dict(check="decision", allowed=["needs_clarification"]), dict(check="no_mutation")]


def unsupported_checks():
    return [dict(check="decision", allowed=["unsupported"]), dict(check="no_mutation")]


def floor(role="syntax.comment", backgrounds=None, min=4.5, semantic=False, persistent=False):
    return action("contrast", role, semantic=semantic, backgrounds=backgrounds or BG,
                  min=min, direction="both", persistent=persistent)


def setup(*actions):
    result = []
    for a in actions:
        entry = dict(op=a["op"], target=target(a["roles"], a["property"], a["include_semantic"]))
        for key, value in a.items():
            if value is None or key in {"op", "roles", "property", "include_semantic"}:
                continue
            mapped = {"number": ("delta" if a["op"] in {"lightness", "chroma"} else "degrees" if a["op"] == "hue" else "value"), "boolean": "value", "underline": "value"}.get(key, key)
            entry[mapped] = value
        result.append(entry)
    return dict(version=1, actions=result)


def metric(kind, role, lo, hi, prop="fg"):
    return check(kind, role, prop, min_delta=lo, max_delta=hi)


def preserved(role, prop="fg", semantic=False):
    return check("preserved", role, prop, semantic)


def permitted_edit(*targets):
    return [only(*targets), locks()]


def font_reset(role):
    return [action("set_bool", role, "bold", boolean=False),
            action("set_bool", role, "italic", boolean=False),
            action("state", role, "underline", state="clear"),
            action("set_bool", role, "strikethrough", boolean=False)]


def base_theme(light=False):
    return {
        "meta": dict(name="Eval paper" if light else "Eval ink", appearance="light" if light else "dark", ir_version=1),
        "styles": {
            "surface.base": dict(bg="#f3eee5" if light else "#151c25", bg_opacity=1),
            "surface.panel": dict(bg="#e4ddd1" if light else "#242e3b"),
            "surface.float": dict(bg="#fffaf1" if light else "#2b3747"),
            "surface.gutter": dict(fg="#a19a90" if light else "#586779"),
            "text.primary": dict(fg="#25201b" if light else "#e5edf6"),
            "text.secondary": dict(fg="#776e63" if light else "#9cabbc"),
            "syntax.comment": dict(fg="#a49b8f" if light else "#586678", italic=True, bold=True, underline="single", strikethrough=True),
            "syntax.keyword": dict(fg="#7054a0" if light else "#bca2ec", italic=True, bold=True),
            "syntax.string": dict(fg="#35734e" if light else "#73c796"),
            "syntax.function": dict(fg="#2e7956" if light else "#70cfa5", bold=True, underline="single", strikethrough=True),
            "syntax.variable": dict(fg="#376d9c" if light else "#87bde2"),
            "syntax.type": dict(fg="#916125" if light else "#e4bd76"),
            "interaction.selection": dict(bg="#394f70", bg_opacity=0.3),
            "interaction.search": dict(bg="#9b7325", bg_opacity=0.3),
            "interaction.search-current": dict(bg="#b38739", bg_opacity=0.65),
            "diagnostic.error.text": dict(fg="#ae453f" if light else "#df8179"),
        },
        "modifiers": [dict(role="syntax.comment", modifiers=["documentation"], token_type="comment", style=dict(italic=True), priority=1),
                      dict(role="syntax.keyword", modifiers=["declaration"], token_type="keyword", style=dict(italic=True, bold=True), priority=2)],
        "extensions": {"vscode.semanticHighlighting": True},
    }


def build():
    (ROOT / "states").mkdir(exist_ok=True)
    for filename, light in [(DARK, False), (LIGHT, True)]:
        (ROOT / filename).write_text(json.dumps(base_theme(light), indent=2) + "\n")
    extra=base_theme(False)
    extra["styles"]["other.unrelated-plugin"]=dict(fg="#bb66cc",bg="#101010",italic=True)
    (ROOT/"states/dark-extra-role.json").write_text(json.dumps(extra,indent=2)+"\n")
    corpus = json.loads((ROOT / "corpus-v1.json").read_text())
    annotations = {c["id"]: c for c in corpus["cases"]}
    cases, responses, pairs = [], {}, []

    def add(id, checks, response, expectation="edit", *, theme=DARK, state=None, context=None, previous=None, source=None, request=None, category=None):
        origin = annotations.get(source or id)
        record = dict(id=id, source_id=(origin["id"] if origin else None), split=(origin["split"] if origin else "development"),
                      category=category or (origin["category"] if origin else "metamorphic"),
                      request=request or origin["request"], theme=theme, checks=checks, expectation=expectation)
        if state is not None: record["setup"] = state
        if context is not None: record["context"] = context
        if previous is not None: record["previous"] = previous
        cases.append(record)
        responses[id] = response

    add("i01", [metric("lightness", "surface.base", -.12, -.005, "bg"), *permitted_edit(target("surface.base", "bg"))], ready(action("lightness", "surface.base", "bg", number=-.04)))
    add("i02", [metric("lightness", "syntax.comment", .005, .15), *permitted_edit(target("syntax.comment"))], ready(action("lightness", "syntax.comment", number=.04)))
    add("i03", [metric("chroma", "syntax.string", -.12, -.005), *permitted_edit(target("syntax.string"))], ready(action("chroma", "syntax.string", number=-.025)))
    add("i04", [check("color", "syntax.function", value="#aa7744"), *permitted_edit(target("syntax.function"))], ready(action("set_color", "syntax.function", color="#aa7744")))
    add("i05", [preserved("surface.base", "bg"), check("contrast", "syntax.comment", backgrounds=BG, min=4.5), only(target("syntax.comment")), locks(target("surface.base", "bg"))], ready(action("lock", "surface.base", "bg", scope="persistent"), floor()))
    add("i06", [check("locked", "syntax.keyword", value=True), check("action_present", "syntax.keyword", op="lock", scope="persistent"), only(), locks(target("syntax.keyword"))], ready(action("lock", "syntax.keyword", scope="persistent")))
    add("i07", [preserved("surface.base", "bg"), check("number", "surface.base", "bg_opacity", value=1), only(target("surface.base", "bg_opacity")), locks(target("surface.base", "bg"))], ready(action("lock", "surface.base", "bg", scope="persistent"), action("set_number", "surface.base", "bg_opacity", number=1)), state=setup(action("set_number", "surface.base", "bg_opacity", number=.4)))
    add("i08", [check("action_present", "syntax.*", op="lock", scope="request"), check("locked", "syntax.*", "fg", value=False), dict(check="no_mutation")], ready(action("lock", "syntax.*", scope="request")))
    add("i09", [metric("lightness", "surface.base", -.12, -.005, "bg"), check("contrast", "syntax.comment", backgrounds=BG, min=4.5), check("bool", "*", "italic", True, value=False), *permitted_edit(target("surface.base", "bg"), target("syntax.comment"), target("*", "italic", True))], ready(action("lightness", "surface.base", "bg", number=-.04), floor(), action("set_bool", "*", "italic", True, boolean=False)))
    add("i10", clarification_checks(), clarify("Do you mean lower selection background opacity and stronger search background, or different foreground treatment?"), "clarification")
    add("i11", [preserved("syntax.function"), metric("chroma", "syntax.keyword", -.12, -.005), only(target("syntax.keyword")), locks(target("syntax.function"))], ready(action("lock", "syntax.function", scope="persistent"), action("chroma", "syntax.keyword", number=-.025)))
    add("i12", [check("color", "syntax.comment", value="#888888"), check("bool", "syntax.comment", "bold", value=False), *permitted_edit(target("syntax.comment"), target("syntax.comment", "bold"))], ready(action("set_color", "syntax.comment", color="#888888"), action("set_bool", "syntax.comment", "bold", boolean=False)))
    for id, question in {"i13":"Which blue role do you mean?", "i14":"Which previous version should be the reference?", "i15":"Do you mean syntax or interaction highlights?", "i18":"What was the previous saturation change?", "i20":"Should the background be black or white?", "i24":"Which previous keyword foreground should be restored?", "i25":"The base surface has no foreground; which role's foreground do you want to unlock?", "i37":"Do you mean neutral temperature or syntax accents?", "i38":"Which red role or color should be protected?", "i39":"Do you mean bold typography or color prominence?", "i44":"What was the previous comment italic setting?", "i45":"What was the previous saturation change?", "i47":"Hue-only protection is not represented; should the whole keyword foreground stay locked?"}.items():
        add(id, clarification_checks(), clarify(question), "clarification")
    add("i16", [metric("lightness", "surface.base", -.04, -.005, "bg"), *permitted_edit(target("surface.base", "bg"))], ready(action("lightness", "surface.base", "bg", number=-.02)))
    add("i17", [metric("lightness", "surface.base", -.16, -.06, "bg"), *permitted_edit(target("surface.base", "bg"))], ready(action("lightness", "surface.base", "bg", number=-.08)))
    add("i19", [dict(check="any_of", alternatives=[clarification_checks(), [outcome("refused"), check("action_present", "syntax.comment", op="lock", scope="persistent"), preserved("syntax.comment"), only(), locks(target("syntax.comment"))]])], clarify("May the comment foreground be changed, or should the protection win?"), "mixed")
    for id, note in {"i21":"Font size is an editor setting, not a theme property.", "i22":"Music playback is outside theme operations.", "i43":"Animated text is not a static theme property."}.items():
        add(id, unsupported_checks(), unsupported(note), "unsupported")
    add("i23", [check("contrast_requested","syntax.comment",backgrounds=BG+SELECTED,min=4.5,persistent=False),check("contrast", "syntax.comment", backgrounds=BG+SELECTED, min=4.5), *permitted_edit(target("syntax.comment"))], ready(floor(backgrounds=BG+SELECTED)), state=setup(action("lock", "surface.base", "bg", scope="persistent")))
    add("i26", [check("locked", "syntax.keyword", value=False), check("action_present", "syntax.keyword", op="unlock", scope=None), only(), locks(target("syntax.keyword"))], ready(action("unlock", "syntax.keyword")), state=setup(action("lock", "syntax.keyword", scope="persistent")))
    add("i27", [check("secondary", "syntax.comment", primary="text.primary", background=BG[0], min=4.5), *permitted_edit(target("syntax.comment"))], ready(floor()))
    add("i28", [check("secondary", "surface.gutter", primary="text.primary", background=BG[0], min=4.5), *permitted_edit(target("surface.gutter"))], ready(floor("surface.gutter")))
    add("i29", [check("bool", "*", "italic", True, value=False), *permitted_edit(target("*", "italic", True))], ready(action("set_bool", "*", "italic", True, boolean=False)))
    add("i30", [check("bool", "syntax.function", "bold", value=True), check("bool", "syntax.keyword", "bold", value=False), *permitted_edit(target("syntax.function", "bold"), target("syntax.keyword", "bold"))], ready(action("set_bool", "syntax.function", "bold", boolean=True), action("set_bool", "syntax.keyword", "bold", boolean=False)), state=setup(action("set_bool", "syntax.function", "bold", boolean=False)))
    add("i31", [metric("lightness", "surface.float", .005, .12, "bg"), *permitted_edit(target("surface.float", "bg"))], ready(action("lightness", "surface.float", "bg", number=.04)))
    add("i32", [check("color", "syntax.string", value="#4f9077"), *permitted_edit(target("syntax.string"))], ready(action("set_color", "syntax.string", color="#4f9077")))
    add("i33", [preserved("interaction.selection", "bg"), check("number", "interaction.selection", "bg_opacity", value=1), only(target("interaction.selection", "bg_opacity")), locks(target("interaction.selection", "bg"))], ready(action("lock", "interaction.selection", "bg", scope="persistent"), action("set_number", "interaction.selection", "bg_opacity", number=1)))
    add("i34", [check("bool", "syntax.comment", "italic", value=True), only(target("syntax.comment", "italic")), locks(*[target("*", p, True) for p in ("fg", "bg", "special")])], ready(*[action("lock", "*", p, True, scope="persistent") for p in ("fg", "bg", "special")], action("set_bool", "syntax.comment", "italic", boolean=True)), state=setup(action("set_bool", "syntax.comment", "italic", boolean=False)))
    add("i35", clarification_checks(), clarify("Should crisp errors mean stronger decoration or a different color contrast floor?"), "clarification")
    add("i36", [check("contrast_requested","syntax.comment",backgrounds=BG+SELECTED,min=4.5,persistent=False),check("bool", "syntax.keyword", "bold", value=False), check("contrast", "syntax.comment", backgrounds=BG+SELECTED, min=4.5), *permitted_edit(target("syntax.keyword", "bold"), target("syntax.comment"))], ready(action("set_bool", "syntax.keyword", "bold", boolean=False), floor(backgrounds=BG+SELECTED)))
    add("i40", [metric("chroma", "syntax.keyword", -.14, -.035), check("hue_preserved", "syntax.keyword", tolerance_degrees=3), check("contrast", "syntax.keyword", backgrounds=BG, min=4.5), *permitted_edit(target("syntax.keyword"))], ready(action("chroma", "syntax.keyword", number=-.04)))
    add("i41", [dict(check="any_of", alternatives=[clarification_checks(), [outcome("refused"), preserved("*", "italic", True)]])], clarify("Should every property remain locked, or may italic styling be changed?"), "mixed")
    add("i42", [outcome("infeasible"), dict(check="no_mutation")], ready(floor(min=7), action("lightness", "syntax.comment", number=-1)), "refusal", state=setup(action("set_color", "surface.base", "bg", color="#000000")))
    add("i46", [check("locked", "interaction.selection", "bg_opacity", value=False), preserved("interaction.selection", "bg"), only(), locks(target("interaction.selection", "bg_opacity"))], ready(action("unlock", "interaction.selection", "bg_opacity")), state=setup(action("lock", "interaction.selection", "bg_opacity", scope="persistent")))
    add("i48", [check("secondary", "diagnostic.error.text", primary="text.primary", background=BG[0], min=4.5), *permitted_edit(target("diagnostic.error.text"))], ready(floor("diagnostic.error.text")), state=setup(action("set_color", "diagnostic.error.text", color="#6b4b49")))
    add("i49", [check("state", "syntax.function", "underline", value="clear"), preserved("syntax.function"), preserved("syntax.function", "bold"), preserved("syntax.function", "strikethrough"), *permitted_edit(target("syntax.function", "underline"))], ready(action("state", "syntax.function", "underline", state="clear")))
    add("i50", [check("bool", "syntax.comment", "bold", value=False), check("bool", "syntax.comment", "italic", value=False), check("state", "syntax.comment", "underline", value="clear"), check("bool", "syntax.comment", "strikethrough", value=False), *permitted_edit(*[target("syntax.comment", p) for p in ("bold", "italic", "underline", "strikethrough")])], ready(*font_reset("syntax.comment")))

    add("i05-already-readable", [preserved("syntax.comment"), outcome("unchanged"), dict(check="no_mutation")], ready(floor()), "mixed", source="i05", state=setup(action("set_color", "syntax.comment", color="#cbd5e1")))
    add("i05-light", [preserved("surface.base", "bg"), check("contrast", "syntax.comment", backgrounds=BG, min=4.5), *permitted_edit(target("syntax.comment"))], ready(floor()), source="i05", theme=LIGHT)
    add("i05-locked", [dict(check="any_of", alternatives=[clarification_checks(), [outcome("infeasible", "refused"), dict(check="no_mutation")]])], ready(floor()), "refusal", source="i05", state=setup(action("lock", "syntax.comment", scope="persistent")))
    add("i05-missing", clarification_checks(), clarify("The comment foreground is absent; should Hexwitch complete it first?"), "clarification", source="i05", state=setup(action("state", "syntax.comment", state="clear")))
    add("i05-alpha", [check("contrast", "syntax.comment", backgrounds=BG, min=4.5), preserved("syntax.comment", "fg_opacity"), *permitted_edit(target("syntax.comment"))], ready(floor()), source="i05", state=setup(action("set_number", "syntax.comment", "fg_opacity", number=.6)))
    add("i01-at-black", [dict(check="any_of", alternatives=[[outcome("unchanged"), dict(check="no_mutation")], [dict(check="decision", allowed=["unchanged"]), dict(check="no_mutation")]])], ready(action("lightness", "surface.base", "bg", number=-.04)), "mixed", source="i01", state=setup(action("set_color", "surface.base", "bg", color="#000000")))
    add("i02-at-white", [outcome("unchanged"), dict(check="no_mutation")], ready(action("lightness", "syntax.comment", number=.04)), "mixed", source="i02", state=setup(action("set_color", "syntax.comment", color="#ffffff")))
    add("i03-neutral", [dict(check="any_of", alternatives=[[outcome("unchanged"), dict(check="no_mutation")], [dict(check="decision", allowed=["unchanged"]), dict(check="no_mutation")]])], dict(version=1,decision="unchanged",notes="Strings are neutral; there is no meaningful chroma to reduce.",questions=[],actions=[]), "mixed", source="i03", state=setup(action("set_color", "syntax.string", color="#888888")))
    add("i29-locked-typography", [outcome("partial"), check("bool", "syntax.comment", "italic", value=True), check("locked", "syntax.comment", "italic", value=True), only(target("*", "italic", True)), locks()], ready(action("set_bool", "*", "italic", True, boolean=False)), "refusal", source="i29", state=setup(action("lock", "syntax.comment", "italic", scope="persistent")))
    add("i23-foreground-replacement", [dict(check="any_of", alternatives=[clarification_checks(), [outcome("infeasible"), dict(check="no_mutation")]])], ready(floor(backgrounds=BG+SELECTED)), "refusal", source="i23", state=setup(action("set_color", "interaction.selection", color="#ffffff")))
    add("i33-already-opaque", [preserved("interaction.selection", "bg"), check("number", "interaction.selection", "bg_opacity", value=1), only(), locks(target("interaction.selection", "bg"))], ready(action("set_number", "interaction.selection", "bg_opacity", number=1)), "mixed", source="i33", state=setup(action("set_number", "interaction.selection", "bg_opacity", number=1)))
    add("i04-locked", [outcome("refused"),dict(check="no_mutation"),check("action_absent","syntax.function",op="unlock")], ready(action("set_color","syntax.function",color="#aa7744")), "refusal", source="i04", state=setup(action("lock","syntax.function",scope="persistent")))
    add("i01-added-role", [metric("lightness","surface.base",-.12,-.005,"bg"),*permitted_edit(target("surface.base","bg"))], copy.deepcopy(responses["i01"]),source="i01",theme="states/dark-extra-role.json")
    add("i05-extra-unrelated", [preserved("surface.float", "bg"), check("contrast", "syntax.comment", backgrounds=BG, min=4.5), *permitted_edit(target("syntax.comment"))], ready(floor()), source="i05", state=setup(action("set_color", "surface.float", "bg", color="#394556")))

    prior=setup(action("chroma", "syntax.keyword", number=-.04))["actions"]
    add("i45-with-context", [metric("chroma", "syntax.keyword", -.027, -.013), *permitted_edit(target("syntax.keyword"))], ready(action("chroma", "syntax.keyword", number=-.02)), source="i45", context=dict(previous_requests=["Reduce keyword chroma by 0.04."],previous_actions=prior))

    add("paraphrase-background", [metric("lightness", "surface.base", -.12, -.005, "bg"), *permitted_edit(target("surface.base", "bg"))], copy.deepcopy(responses["i01"]), request="Please darken only the editor background.")
    pairs.append(dict(check="equivalent",id="unrelated-role-invariance",left="i01",right="i01-added-role",targets=[target("surface.base","bg")]))
    pairs.extend([dict(check="equivalent",id="background-paraphrase",left="i01",right="paraphrase-background",targets=[target("surface.base","bg")]),
                  dict(check="strength_ordered",id="slightly-versus-much",smaller="i16",larger="i17",target=target("surface.base","bg"),metric="lightness",min_gap=.05)])

    shared_setup=setup(action("set_number", "surface.base", "bg_opacity", number=.5))
    add("seq-lock", [check("locked", "surface.base", "bg", value=True), only(), locks(target("surface.base","bg"))], ready(action("lock","surface.base","bg",scope="persistent")), request="Lock only the background color.",category="iteration",state=shared_setup)
    add("seq-opacity", [check("number","surface.base","bg_opacity",value=1), preserved("surface.base","bg"),check("locked","surface.base","bg",value=True),only(target("surface.base","bg_opacity")),locks()], ready(action("set_number","surface.base","bg_opacity",number=1)), request="Now remove background transparency without unlocking its color.",category="iteration",previous="seq-lock")
    add("seq-readable", [check("contrast_requested","syntax.comment",backgrounds=BG,min=4.5,persistent=True),check("contrast","syntax.comment",backgrounds=BG,min=4.5),preserved("surface.base","bg"),only(target("syntax.comment")),locks()], ready(floor(persistent=True)), request="Make comments readable and keep this contrast requirement for future edits.",category="iteration",previous="seq-opacity")
    add("seq-refused", [outcome("refused"),dict(check="no_mutation"),check("action_absent","surface.base","bg",op="unlock")], ready(action("lightness","surface.base","bg",number=.04)),request="Make the background lighter.",category="iteration",previous="seq-readable",expectation="refusal")
    add("seq-unlock", [check("locked","surface.base","bg",value=False),metric("lightness","surface.base",-.1,-.005,"bg"),check("contrast","syntax.comment",backgrounds=BG,min=4.5),only(target("surface.base","bg")),locks(target("surface.base","bg"))],ready(action("unlock","surface.base","bg"),action("lightness","surface.base","bg",number=-.04)),request="Explicitly unlock only the background color and make it darker.",category="iteration",previous="seq-refused")
    add("seq-idempotent", [outcome("unchanged"),dict(check="no_mutation")],ready(floor(persistent=True)),request="Make comments readable again with the same persistent contrast floor.",category="iteration",previous="seq-unlock",expectation="mixed")
    add("seq-local-lock", [check("action_present","syntax.keyword",op="lock",scope="request"),outcome("refused"),dict(check="no_mutation")],ready(action("lock","syntax.keyword",scope="request"),action("lightness","syntax.keyword",number=.04)),request="Keep keyword foreground during this request, even if a change is proposed.",category="iteration",expectation="refusal")
    add("seq-local-released", [metric("lightness","syntax.keyword",.005,.1),*permitted_edit(target("syntax.keyword"))],ready(action("lightness","syntax.keyword",number=.04)),request="Make keywords brighter on this next request.",category="iteration",previous="seq-local-lock")

    assert {c["id"] for c in cases if c["id"] in annotations} == set(annotations)
    suite=dict(version=1,origin="Hexwitch authored semantic state matrix",license="MIT",revision="eval-matrix-1",purpose="All 50 corpus requests plus state variants, sequential preservation and paired consistency",
               limitations=["Original held_out labels retained, but cases inspected during implementation are not a fresh independent holdout", "Canned responses test the evaluator, not model quality", "Secondary/hue/chroma checks are numerical proxies, not human preference", "Some relational/context requests require clarification; edit/clarification/unsupported scores are separate"],cases=cases,pairs=pairs)
    run=dict(version=1,label="Authored eval-matrix contract replay",provider="offline",model="none",prompt_version="property-intent-1",kind="canned_replay",responses=[dict(case_id=c["id"],response=responses[c["id"]]) for c in cases])
    (ROOT/"interpretation-matrix-v1.json").write_text(json.dumps(suite,indent=2)+"\n")
    (ROOT/"interpretation-matrix-replay.json").write_text(json.dumps(run,indent=2)+"\n")
    print(f"Authored {len(cases)} cases and {len(pairs)} pair checks; all 50 corpus IDs covered")


if __name__ == "__main__":
    build()
