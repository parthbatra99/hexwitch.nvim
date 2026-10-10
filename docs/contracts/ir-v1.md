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

`modifiers` is a list of `{role, modifiers, language?, style}`; relationships are
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
An action target is `{roles:[validated selectors], property:"bg_opacity"}`.
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
    {"op":"set_bool","target":{"roles":["syntax.*"],"property":"italic"},"value":false}
  ]
}
```

`requirements` specify final role/property/backgrounds/min/persistent. A failed
required constraint rolls back the v1 candidate transaction. Persistent
constraints are stored under `hexwitch.constraints.v1`; literal backgrounds do
not follow later surface edits (role-aware constraints are future work).
`preferences` have a target, weight 0..1, reason; they are reported as soft
identity preferences, not yet optimized. Outcomes are distinct from parse errors.

## Protocol

`hexwitch --machine --request-id my-request capabilities` discovers versions.
Results contain protocol_version, request_id, operation, outcome, result,
errors (stable code + message), warnings. Domain partial/refused/infeasible
outcomes are successful transport responses; malformed/IO/provider errors exit 1.
Clap argument errors occur before protocol execution and remain ordinary CLI errors.
Artifact replacement is atomic per file, not one transaction across all outputs.

## Precision and validation

Contrast works on rounded sRGB output. Fixed-hue gamut-mapped lightness search
samples 4096 points per direction and refines the first observed crossing 24
iterations; it is bounded numerical search, not proof of global minimality.
Recovered low-chroma 8-bit hue is allowed 1 degree rounding tolerance in tests.
Scene ordering must be explicitly verified to pass overlays. Default scene
reports are calculated profiles, not live editor resolution/screenshot evidence.
