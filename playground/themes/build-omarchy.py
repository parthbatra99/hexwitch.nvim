import argparse
import hashlib
import json
import re
import sys
import tomllib
from pathlib import Path

ORIGIN = "https://github.com/basecamp/omarchy"
TEMPLATE_PATH = "default/themed/vscode-theme.json.tpl"
NAMES = {"rose-pine": "Rosé Pine", "retro-82": "Retro 82", "flexoki-light": "Flexoki Light"}
TOKEN = re.compile(r"\{\{\s*([a-z_]+)\s*\}\}")
MIX = re.compile(r"\{\{\s*mix\s+([A-Za-z0-9_]+)\s+([A-Za-z0-9_]+)\s+([0-9]+(?:\.[0-9]+)?)%?\s*\}\}")
HEX = re.compile(r"^#[0-9a-fA-F]{6}$")
ROLE_SCOPES = [
    "comment",
    "keyword",
    "keyword.operator",
    "string",
    "constant.numeric",
    "constant.language",
    "constant.character.escape",
    "entity.name.function",
    "entity.name.type",
    "variable",
    "variable.parameter",
    "variable.other.property",
    "variable.language",
    "entity.name.tag",
    "entity.other.attribute-name",
    "punctuation",
]


def mix(start, end, amount):
    a = [int(start[i:i + 2], 16) for i in (1, 3, 5)]
    b = [int(end[i:i + 2], 16) for i in (1, 3, 5)]
    return "#" + "".join(f"{int(x * (1 - amount) + y * amount + 0.5):02x}" for x, y in zip(a, b))


def resolve(raw):
    colors = {key: value.lower() for key, value in raw.items() if isinstance(value, str) and HEX.match(value)}
    colors.setdefault("muted", colors.get("dark_foreground", colors["foreground"]))
    colors.setdefault("dark_foreground", colors["foreground"])
    colors.setdefault("bright_foreground", colors["foreground"])
    colors.setdefault("selection", colors.get("selection_background", colors["background"]))
    colors.setdefault("selection_background", colors["selection"])
    colors.setdefault("selection_foreground", colors["bright_foreground"])
    colors.setdefault("orange", colors["yellow"])
    for hue in ("red", "yellow", "green", "cyan", "blue", "magenta"):
        colors.setdefault(f"bright_{hue}", mix(colors[hue], "#ffffff", 0.2))
    mode = raw.get("mode", "dark")
    if mode not in ("dark", "light"):
        raise ValueError(f"unsupported mode {mode!r}")
    colors["theme_type"] = mode
    return colors


def render(template, colors, slug):
    def substitute(match):
        name = match.group(1)
        if name not in colors:
            raise KeyError(f"{slug}: template needs {name}")
        return colors[name]

    def blend(match):
        start, end, amount = match.groups()
        for name in (start, end):
            if name not in colors:
                raise KeyError(f"{slug}: template mix needs {name}")
        value = float(amount)
        return mix(colors[start], colors[end], value / 100 if value > 1 or "%" in match.group(0) else value)

    rendered = TOKEN.sub(substitute, MIX.sub(blend, template))
    if "{{" in rendered:
        raise ValueError(f"{slug}: unsupported template expression {rendered[rendered.index('{{'):][:60]}")
    return json.loads(rendered)


def known_keys(ingest_source):
    return set(re.findall(r'"([A-Za-z]+(?:\.[A-Za-z0-9]+)+)"', ingest_source))


def role_rules(token_colors):
    rules = []
    for scope in ROLE_SCOPES:
        for rule in token_colors:
            scopes = rule["scope"] if isinstance(rule["scope"], list) else [rule["scope"]]
            if scope in scopes:
                rules.append({"scope": scope, "settings": rule["settings"]})
                break
    return rules


def excerpt(theme, keys, name):
    colors = {key: value for key, value in theme["colors"].items() if key in keys or key.startswith("terminal.ansi")}
    return {
        "name": name,
        "type": theme["type"],
        "colors": colors,
        "tokenColors": role_rules(theme["tokenColors"]),
        "semanticTokenColors": theme["semanticTokenColors"],
    }


def display_name(slug):
    return NAMES.get(slug) or " ".join(part.capitalize() for part in slug.split("-"))


def build(args):
    template = Path(args.template).read_text()
    keys = known_keys(Path(args.ingest).read_text())
    themes = []
    for path in sorted(Path(args.colors).glob("*.toml")):
        slug = path.stem
        data = path.read_bytes()
        colors = resolve(tomllib.loads(data.decode()))
        name = display_name(slug)
        rendered = render(template, colors, slug)
        themes.append({
            "id": f"omarchy-{slug}",
            "name": name,
            "mode": colors["theme_type"],
            "upstream_path": f"themes/{slug}/colors.toml",
            "colors_sha256": hashlib.sha256(data).hexdigest(),
            "dots": [colors["background"], colors["bright_magenta"], colors["green"], colors["yellow"]],
            "theme": excerpt(rendered, keys, f"{name} (Omarchy)"),
        })
    bundle = {
        "version": 1,
        "source": {
            "name": "Omarchy",
            "origin": ORIGIN,
            "revision": args.revision,
            "license": "MIT",
            "notice": "OMARCHY-LICENSE",
            "template": TEMPLATE_PATH,
            "template_sha256": hashlib.sha256(template.encode()).hexdigest(),
            "method": "Curated excerpt: each theme's colors.toml rendered through Omarchy's VS Code template, keeping the workbench keys the engine imports, the semantic token colors, and the first template rule for each generic syntax scope.",
        },
        "themes": themes,
    }
    Path(args.out).write_text(json.dumps(bundle, ensure_ascii=False, separators=(",", ":")) + "\n")
    print(f"wrote {len(themes)} themes to {args.out}")


def main():
    parser = argparse.ArgumentParser(description="Bundle Omarchy's official themes for the playground.")
    parser.add_argument("--colors", required=True, help="directory of <theme>.toml files copied from themes/<theme>/colors.toml")
    parser.add_argument("--template", required=True, help=f"copy of {TEMPLATE_PATH} at the same revision")
    parser.add_argument("--revision", required=True, help="upstream commit the files came from")
    parser.add_argument("--ingest", default=str(Path(__file__).resolve().parents[2] / "core/crates/hexwitch-core/src/ingest.rs"))
    parser.add_argument("--out", default=str(Path(__file__).with_name("omarchy.json")))
    build(parser.parse_args())


if __name__ == "__main__":
    sys.exit(main())
