# IR and intent v1

## Theme

`meta.ir_version` must be `1`. Unknown top-level and style metadata fields are
rejected. Call `Theme::from_json_str`/`load_theme` and `Theme::validate` at
boundaries; a standalone `Style` still needs `Style::validate`. Direct serde
deserialization of `Theme` also validates versions and styles. Newer versions are
rejected, and nothing is converted forward implicitly. The only migration path is
explicit: `hexwitch import old.json --migrate-v0 --out migrated.json`, which turns
whole-style locked provenance into per-property locks.

The `fg`, `bg`, `special`, `bold`, `italic`, `underline` and `blend` slots carry
over from v0. `fg_opacity`, `bg_opacity` and `special_opacity` are finite 0..1,
and omission means opaque. `blend` is 0..100. `states` maps property names to
`unspecified`, `inherit` or `clear`. Without a state, a property with a concrete
slot is `value` and one without is unspecified; a state plus a concrete slot is
invalid. `false` is a concrete typography reset, not unspecified. Clearing a
property leaves its opacity and lock alone.

`locked` is a set of property names. `source` has an origin (`imported`,
`generated`, `derived` or `user`) and an optional reference. The v0 `provenance`
field is still read but is deprecated for new protection writes.

`strikethrough` is its own optional bool, so a rule can have both underline and
strikethrough. The legacy `underline:"strikethrough"` is still accepted.

`modifiers` is a list of `{role, modifiers, language?, style, token_type?, priority?}`.
`token_type` keeps class/interface/method distinctions and `priority` records
source order (default 0). An empty modifier set expresses an unmodified type or
language rule. Rules score type +100, each modifier +100 and language +10; ties
go to the later priority or list position, per property. Semantic enablement lives
in `extensions["vscode.semanticHighlighting"]`, and unknown enablement is reported
as qualified. Older engines may reject these additive fields, so check
`capabilities.ir_features` before writing them.

Relationships are `{from,to,kind}` with kind `inherits`, `surface`, `association`
or `lower_prominence`. Both endpoints must exist and inheritance cannot cycle.
`extensions` is a namespaced JSON passthrough map. Keeping an extension does not
mean another editor can execute what it describes.

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

## Semantic import

The selector grammar is `(*|type)(.modifier)*(:language)?`, resolved per property
with the scoring above. Includes keep ordered semantic layers, and parent/child
enablement is ORed as VS Code does. Duplicate JSON object keys, malformed
selectors, colors, bools or fontStyle, and unmapped settings are rejected.
Machine import returns `partial` when the source loses information.

The rules were checked against VS Code 1.95.0
(`912bb683695358a54ae0c670461738984cbb5b95`), mainly
`src/vs/platform/theme/common/tokenClassificationRegistry.ts` and
`src/vs/workbench/services/themes/common/colorThemeData.ts`. The Rust code is
written locally, not copied. Pinned theme files and their MIT notices are listed
in `core/eval/import/manifest.json`.

This is not TextMate parsing. Exclusions, unions, priority predicates, grammars
and editor defaults are outside the supported scope-probe grammar, and multiple
lexical scopes collapse into one reported representative role. Custom semantic
type hierarchies need extension context the engine does not have. None of this
has been checked against a running editor or language server.

## Intent

`--machine revise` treats a spec as v1 when it has `version:1` and `actions`.
Legacy `protect`/`changes` specs are still accepted but are not property-exact.
An action target is `{roles:[selectors], property:"bg_opacity", include_semantic?:false}`.
With `include_semantic` true the action also applies to every existing semantic
override, and locks, refusals and reports keep the rule's identity. Selectors are
exact present roles, `*`, known namespace prefixes and known kind prefixes; any
other string ending in `.*` is invalid.

Operations: `lock` (scope `request` or `persistent`), `unlock`, `set_color`,
`set_number`, `set_bool`, `set_underline`, `state`, `lightness`, `chroma`, `hue`,
and `contrast` with `backgrounds`, `min` and direction.

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

`requirements` give a final role, property, backgrounds, min and persistence, with
optional `include_semantic`. A background is a literal hex or
`{surface:"surface.base",overlays:[]}`; role backgrounds follow later surface
changes and compose alpha. A background-only floor cannot verify overlays that
replace the foreground or blend with something external. If a required constraint
fails, the whole v1 candidate rolls back and `report.rolled_back` is true.
Persistent constraints are stored under `hexwitch.constraints.v1`, and adding the
same requirement twice is a no-op.

Semantic floors check the combined effect of applicable modifier and language
rules, not each rule alone. Above 256 contexts or eight modifiers the check is
unverified and fails closed. Reports name the failing `semantic_context`.
`preferences` (target, weight 0..1, reason) are reported but not yet optimized.
Outcomes are separate from parse errors.

## Interpretation v1

`interpret --v1` is opt-in, and legacy replay still works. A model response has
`version`, `decision`, `notes`, `questions` and `actions`. Each action names a
role collection, property, `include_semantic` and every operation field, using
null for fields it does not use. The provider schema has one branch per operation:
property enums match the operation, color deltas, hue and contrast have numeric
bounds, and opacity and integer blend are separate branches. The `set_color`
branch only allows colors the user typed in the request, and is removed when there
are none. Unknown or missing fields and non-null unused fields fail validation.

`ready` needs actions. `unchanged`, `needs_clarification` and `unsupported` must
have none, and clarification needs questions. Stdout is the interpretation and
report; `--out` writes an IntentSpec only for actionable responses and leaves
existing files alone otherwise. Replay still needs the original request so user
hex anchors can be checked without trusting colors the model produced. `--context`
supplies reviewed `previous_requests`/`previous_actions`, never inferred history.
`--dry-run --v1` prints the exact prompts without calling the network.

## Evaluation

`eval-interpretation` scores recorded responses against executable semantic
predicates, with SHA-256 hashes for reproducibility and categorized failures. A
canned replay proves the contract works, not that a model is good.

Cases accept optional `setup` (an authored IntentSpec), `previous` (the actual
result of an earlier turn), `source_id` and `expectation`. Suites can declare
`pairs` that check property equivalence or ordered strength. An `any_of`
alternative passes only if one whole branch passes. Operation predicates tell an
unsafe proposed unlock apart from one that was rolled back. Reports include
`pair_failures`, evaluated and `not_evaluated` counts, `upstream_failed` and
separate buckets per expectation. A pair failure fails the run even when every
case passes. The original 12-case format is still accepted. See
[core/eval/README.md](../../core/eval/README.md) for the suites.

## Protocol

`hexwitch --machine --request-id my-request capabilities` lists supported
versions. Results contain `protocol_version`, `request_id`, `operation`,
`outcome`, `result`, `errors` (stable code and message) and `warnings`. Partial,
refused and infeasible outcomes are successful responses; malformed input, IO and
provider errors exit 1. Clap argument errors happen before the protocol runs and
stay ordinary CLI errors. Each artifact file is replaced atomically, private at
creation, and symbolic links and colliding output paths are rejected. There is no
transaction across several output files.

## API connection

Live interpretation uses OpenAI-compatible Chat Completions only. `--base-url` (or
`OPENAI_BASE_URL`) is the API root, and `/chat/completions` is appended with no
`/v1` added. URLs with credentials, query strings, fragments or a full endpoint
path are rejected. `--api-key-env` names the environment variable holding the key
(default `OPENAI_API_KEY`); a literal key is never accepted on the command line.
Legacy `--provider` aliases only set defaults. Clients discover profiles through
`capabilities.provider_transport`.

`--response-format` is `json_schema`, `json_object` or `none`. Local validation is
equally strict in every mode, and the non-schema modes put the schema in the
prompt. HTTP and HTTPS both work; HTTP prints a warning because credentials and
prompts travel unencrypted. `curl` receives the key and payload on stdin, never in
argv or a file, and does not follow redirects. Payload, response size and time
limits apply, and transport errors are redacted. Nothing is retried, downgraded or
given extra sampling parameters automatically.

## Precision and validation

Contrast is measured on rounded sRGB output. The fixed-hue, gamut-mapped lightness
search samples 4096 points per direction and refines the first crossing for 24
iterations. That is a bounded search, not a proof of the global minimum. Tests
allow 1 degree of rounding error on low-chroma 8-bit hues. Scene ordering has to
be verified explicitly before overlays pass. Default scene reports are calculated
profiles, not evidence from a running editor or a screenshot.
