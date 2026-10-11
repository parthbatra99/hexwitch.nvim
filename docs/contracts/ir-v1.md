# IR and intent v1 compatibility

## Theme

`meta.ir_version` must explicitly be `1`. Unknown top-level/style metadata fields
are rejected. Call `Theme::from_json_str`/`load_theme` and `Theme::validate` at
boundaries; standalone Style deserialization still requires Style::validate.
Direct Theme serde deserialization also validates versions and styles.
`hexwitch import old.json --migrate-v0 --out migrated.json` is the explicit v0 path.
Older whole-style locked provenance is converted to independent property locks.
Newer versions are rejected; no implicit forward conversion is attempted.

Existing `fg`, `bg`, `special`, `bold`, `italic`, `underline`, `blend` slots remain.
`fg_opacity`, `bg_opacity`, `special_opacity` are finite 0..1; omission means opaque.
`blend` is 0..100. `states` maps property names to `unspecified`, `inherit`, or
`clear`; omission with a concrete slot is `value`, omission without one is
unspecified. State + concrete slot is invalid. `false` is a concrete typography
reset, not unspecified. Clearing a property does not clear its opacity or lock.

`locked` is an independent property set. `source` has origin `imported`,
`generated`, `derived`, or `user`, and optional reference. Legacy `provenance`
remains for v0 compatibility and is deprecated for new protection writes.

`strikethrough` is an independent optional bool property; new rules can keep both
underline and strike. Legacy `underline:"strikethrough"` remains accepted.

`modifiers` is a list of `{role, modifiers, language?, style, token_type?, priority?}`.
Optional token_type retains class/interface/method distinctions; priority records
source order (default 0). Empty modifier sets express unmodified type/language
rules. Standard semantic rule scores are type +100, modifier +100 each, language
+10; ties resolve later priority/list order per property. Enablement is retained
under `extensions["vscode.semanticHighlighting"]`; unknown enablement is qualified.
Older engines may reject additive fields; check capabilities.ir_features before
writing them. Relationships are
`{from,to,kind}` where kind is `inherits`, `surface`, `association`, or
`lower_prominence`. Both endpoints must exist; inheritance cycles are invalid.
`extensions` is an explicit namespaced JSON passthrough map. Preservation does
not imply that another editor can execute those source semantics.

```json
{
  "meta": {"name":"selection example","appearance":"dark","ir_version":1},
  "styles": {
    "interaction.selection": {
      "fg":"#eeeeee", "bg":"#224466", "bg_opacity":0.4,
      "italic":false, "locked":["fg"],
      "source":{"origin":"generated"}
    }
  }
}
```

## Intent

`--machine revise` detects v1 by required `version:1`, `actions`. Legacy
`protect`/`changes` specs remain accepted but are not property-exact.
An action target is `{roles:[validated selectors], property:"bg_opacity",
include_semantic?:false}`. Setting include_semantic true also targets all existing
semantic override styles; locks/refusals and reports retain their rule identity.
Selectors are exact present roles, `*`, known namespace prefixes, and known
kind prefixes; arbitrary strings ending in `.*` do not validate.

Operations: `lock` (scope `request` or `persistent`), `unlock`, `set_color`,
`set_number`, `set_bool`, `set_underline`, `state`, `lightness`, `chroma`, `hue`,
`contrast` with explicit literal `backgrounds`, `min`, and direction.

```json
{
  "version":1,
  "actions":[
    {"op":"lock","target":{"roles":["surface.base"],"property":"bg"}},
    {"op":"set_number","target":{"roles":["surface.base"],"property":"bg_opacity"},"value":1},
    {"op":"set_bool","target":{"roles":["*"],"property":"italic","include_semantic":true},"value":false}
  ]
}
```

`requirements` specify final role/property/backgrounds/min/persistent, with
optional include_semantic. Backgrounds accept literal hex or
`{surface:"surface.base",overlays:[]}`; role scenes follow later surface changes
and compose alpha. Foreground-replacing overlays/external blending cannot be
verified by a background-only floor. A failed required constraint rolls back the
v1 candidate transaction and sets report.rolled_back=true. Persistent constraints
are stored under `hexwitch.constraints.v1`; duplicate requirements are idempotent.
Semantic floors check bounded effective combinations, not only independent rules;
more than 256 contexts/eight modifiers is unverified and fails closed. Intent
reports identify failed semantic_context and mark proposed edits rolled_back.
`preferences` have a target, weight 0..1, reason; they are reported as soft
identity preferences, not yet optimized. Outcomes are distinct from parse errors.

## Interpretation v1

`interpret --v1` is opt-in; legacy replay remains supported. A model response has
version, decision, notes, questions, actions. Every action names role collection,
property, include_semantic and required nullable operation fields. Unknown/missing
fields and unused non-null parameters fail validation. ready requires actions;
unchanged/needs_clarification/unsupported require no actions. Clarification needs
questions. CLI stdout returns the interpretation/report; `--out` writes only an
IntentSpec for actionable responses and leaves existing artifacts unchanged for
clarification. Original request is required even for replay so opaque user hex
anchors can be validated without trusting model-generated color values.

`--context` provides reviewed previous_requests/previous_actions, not inferred
history. `eval-interpretation` judges recorded responses against executable semantic
predicates with SHA-256 reproducibility and categorized failures. A canned replay
is contract evidence, not a live model-quality benchmark.

Evaluation v1 now accepts optional setup (authored IntentSpec), previous (earlier
actual turn result), source_id and expectation. Optional pairs check property
equivalence or ordered strength. any_of alternatives require one whole branch to
pass; operation predicates distinguish an unsafe proposed unlock from rollback.
Reports expose pair_failures, checks/evaluated and not_evaluated denominators,
upstream_failed, and separate expectation buckets. Pair-only failures must qualify
application/result even if isolated case counts are green. The original 12-case
format remains accepted; feature discovery names the extended evaluator.

## Protocol

`hexwitch --machine --request-id my-request capabilities` discovers versions.
Results contain protocol_version, request_id, operation, outcome, result,
errors (stable code + message), warnings. Domain partial/refused/infeasible
outcomes are successful transport responses; malformed/IO/provider errors exit 1.
Clap argument errors occur before protocol execution and remain ordinary CLI errors.
Artifact replacement is atomic per file, not one transaction across all outputs.

## API connection

Live interpretation uses OpenAI-compatible Chat Completions only. --base-url
(or OPENAI_BASE_URL) selects the API root; /chat/completions is appended.
--api-key-env names the credential environment variable (default OPENAI_API_KEY),
never accepts a literal key. --response-format is json_schema/json_object/none;
validation is strict in every mode, and non-schema modes include schema guidance
in the prompt. Remote URLs require HTTPS; local loopback HTTP is supported.
Legacy --provider aliases set convenience defaults only. Clients discover profiles
via capabilities.provider_transport. No credential-file discovery or automatic
network format retry is performed.

## Precision and validation

Contrast works on rounded sRGB output. Fixed-hue gamut-mapped lightness search
samples 4096 points per direction and refines the first observed crossing 24
iterations; it is bounded numerical search, not proof of global minimality.
Recovered low-chroma 8-bit hue is allowed 1 degree rounding tolerance in tests.
Scene ordering must be explicitly verified to pass overlays. Default scene
reports are calculated profiles, not live editor resolution/screenshot evidence.
