//! Neovim colorscheme emission.
//!
//! Strategy:
//! * Define a base layer of standard highlight groups directly from IR
//!   roles (`Normal`, `Comment`, `String`, `DiagnosticError`, ...).
//! * Link the ecosystem layer on top: Tree-sitter captures (`@comment`,
//!   `@keyword.operator`), LSP semantic groups (`@lsp.type.*`), and
//!   fallback aliases (`Conditional` → `Keyword`). Links — not duplicate
//!   definitions — so user overrides and downstream plugins keep working.
//! * Terminal palette via `vim.g.terminal_color_0..15`.
//!
//! Everything goes through `vim.api.nvim_set_hl`, so cterm fallbacks are
//! Neovim's problem, not ours.

use std::collections::HashSet;
use std::fmt::Write as _;
use thiserror::Error;

use crate::color::Srgb;
use crate::ir::role::{
    DiagnosticKind, DiagnosticPart, DiffKind, DiffPart, InteractionRole, Role, Surface, SyntaxRole,
    TextRole,
};
use crate::ir::style::{Style, UnderlineStyle};
use crate::ir::theme::{Appearance, Theme};

#[derive(Debug, Error, PartialEq)]
pub enum EmitError {
    #[error("theme `{0}` has no surface.base background; cannot emit")]
    MissingBase(String),
    #[error("theme `{0}` has no text.primary foreground; cannot emit")]
    MissingPrimary(String),
    #[error("invalid theme: {0}")]
    InvalidTheme(String),
}

/// What the adapter could and could not do. Deliberately public: callers
/// (CLI, tests, the future plugin) surface this to the user instead of
/// letting silent degradation pass.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct EmitReport {
    pub defined: Vec<String>,
    pub linked: Vec<String>,
    /// Groups skipped because the IR lacks the role (with the missing role).
    pub skipped: Vec<(String, String)>,
    /// IR roles this target has no mapping for (yet).
    pub unconsumed_roles: Vec<String>,
    /// Property-specific degradation, including approximate alpha lowering.
    pub losses: Vec<(String, String, String)>,
}

pub struct NvimEmit {
    pub lua: String,
    pub report: EmitReport,
}

/// Which parts of a role's style feed a group.
enum Part {
    /// fg (+ font attributes) from the role.
    Fg(Role),
    /// bg (+ blend) from the role.
    Bg(Role),
    /// Everything from the role.
    All(Role),
}

fn r_surface(s: Surface) -> Role {
    Role::Surface(s)
}
fn r_text(t: TextRole) -> Role {
    Role::Text(t)
}
fn r_syntax(s: SyntaxRole) -> Role {
    Role::Syntax(s)
}
fn r_interact(i: InteractionRole) -> Role {
    Role::Interaction(i)
}
fn r_diag(kind: DiagnosticKind, part: DiagnosticPart) -> Role {
    Role::Diagnostic { kind, part }
}
fn r_diff(kind: DiffKind, part: DiffPart) -> Role {
    Role::Diff { kind, part }
}

/// Base groups defined directly from roles: (group, sources, fallback link).
/// `Part::Fg` also carries the source style's font attributes.
fn base_definitions() -> Vec<(&'static str, Vec<Part>, Option<&'static str>)> {
    use DiagnosticKind as DK;
    use DiagnosticPart as DP;
    use DiffKind as XK;
    use DiffPart as XP;
    use InteractionRole as IR;
    use Surface as S;
    use SyntaxRole as X;
    use TextRole as T;

    let entries: Vec<(&'static str, Vec<Part>, Option<&'static str>)> = vec![
        // Core editor
        (
            "Normal",
            vec![Part::Fg(r_text(T::Primary)), Part::Bg(r_surface(S::Base))],
            None,
        ),
        ("NormalNC", vec![], Some("Normal")),
        (
            "NormalFloat",
            vec![
                Part::Fg(r_text(T::Secondary)),
                Part::Bg(r_surface(S::Float)),
            ],
            Some("Normal"),
        ),
        (
            "FloatBorder",
            vec![
                Part::Fg(r_surface(S::Border)),
                Part::Bg(r_surface(S::Float)),
            ],
            None,
        ),
        (
            "Pmenu",
            vec![
                Part::Fg(r_text(T::Secondary)),
                Part::Bg(r_surface(S::Float)),
            ],
            Some("Normal"),
        ),
        (
            "PmenuSel",
            vec![
                Part::Fg(r_text(T::Primary)),
                Part::Bg(r_interact(IR::Selection)),
            ],
            None,
        ),
        ("WinSeparator", vec![Part::Fg(r_surface(S::Border))], None),
        (
            "StatusLine",
            vec![
                Part::Fg(r_text(T::Primary)),
                Part::Bg(r_surface(S::Statusline)),
            ],
            Some("Normal"),
        ),
        (
            "StatusLineNC",
            vec![
                Part::Fg(r_text(T::Muted)),
                Part::Bg(r_surface(S::Statusline)),
            ],
            Some("StatusLine"),
        ),
        (
            "TabLine",
            vec![Part::Fg(r_text(T::Muted)), Part::Bg(r_surface(S::Tabline))],
            Some("Normal"),
        ),
        (
            "TabLineSel",
            vec![
                Part::Fg(r_text(T::Primary)),
                Part::Bg(r_surface(S::Tabline)),
            ],
            Some("Normal"),
        ),
        (
            "TabLineFill",
            vec![Part::Bg(r_surface(S::Tabline))],
            Some("Normal"),
        ),
        ("LineNr", vec![Part::Fg(r_surface(S::Gutter))], None),
        (
            "CursorLineNr",
            vec![Part::Fg(r_text(T::Secondary))],
            Some("LineNr"),
        ),
        ("SignColumn", vec![Part::Bg(r_surface(S::Gutter))], None),
        (
            "FoldColumn",
            vec![Part::Fg(r_text(T::Muted)), Part::Bg(r_surface(S::Gutter))],
            Some("LineNr"),
        ),
        (
            "Folded",
            vec![
                Part::Fg(r_text(T::Secondary)),
                Part::Bg(r_surface(S::Panel)),
            ],
            Some("Normal"),
        ),
        ("NonText", vec![Part::Fg(r_text(T::Muted))], None),
        ("EndOfBuffer", vec![], Some("NonText")),
        ("Whitespace", vec![], Some("NonText")),
        ("Conceal", vec![Part::Fg(r_text(T::Muted))], None),
        ("Title", vec![Part::Fg(r_text(T::Accent))], None),
        // Cursor / selection / search
        ("Cursor", vec![Part::Bg(r_interact(IR::Cursor))], None),
        (
            "CursorLine",
            vec![Part::Bg(r_interact(IR::CursorLine))],
            None,
        ),
        ("CursorColumn", vec![], Some("CursorLine")),
        ("Visual", vec![Part::Bg(r_interact(IR::Selection))], None),
        (
            "VisualNC",
            vec![Part::Bg(r_interact(IR::SelectionInactive))],
            Some("Visual"),
        ),
        ("Search", vec![Part::All(r_interact(IR::Search))], None),
        (
            "CurSearch",
            vec![Part::All(r_interact(IR::SearchCurrent))],
            None,
        ),
        ("IncSearch", vec![], Some("CurSearch")),
        (
            "MatchParen",
            vec![Part::All(r_interact(IR::MatchParen))],
            None,
        ),
        // Standard syntax groups
        ("Comment", vec![Part::All(r_syntax(X::Comment))], None),
        ("Constant", vec![Part::All(r_syntax(X::Constant))], None),
        ("String", vec![Part::All(r_syntax(X::String))], None),
        ("Character", vec![], Some("String")),
        ("Number", vec![Part::All(r_syntax(X::Number))], None),
        ("Float", vec![], Some("Number")),
        (
            "Boolean",
            vec![Part::All(r_syntax(X::Boolean))],
            Some("Constant"),
        ),
        ("Identifier", vec![Part::All(r_syntax(X::Variable))], None),
        ("Function", vec![Part::All(r_syntax(X::Function))], None),
        ("Keyword", vec![Part::All(r_syntax(X::Keyword))], None),
        ("Statement", vec![], Some("Keyword")),
        ("Conditional", vec![], Some("Keyword")),
        ("Repeat", vec![], Some("Keyword")),
        ("Label", vec![], Some("Keyword")),
        ("Exception", vec![], Some("Keyword")),
        (
            "Operator",
            vec![Part::All(r_syntax(X::Operator))],
            Some("Keyword"),
        ),
        (
            "PreProc",
            vec![Part::All(r_syntax(X::Macro))],
            Some("Keyword"),
        ),
        ("Include", vec![], Some("PreProc")),
        ("Define", vec![], Some("Keyword")),
        ("Macro", vec![], Some("PreProc")),
        ("PreCondit", vec![], Some("PreProc")),
        ("Type", vec![Part::All(r_syntax(X::Type))], None),
        ("StorageClass", vec![], Some("Type")),
        ("Structure", vec![], Some("Type")),
        ("Typedef", vec![], Some("Type")),
        ("Special", vec![Part::All(r_syntax(X::Escape))], None),
        ("SpecialChar", vec![], Some("Special")),
        ("Tag", vec![Part::All(r_syntax(X::Tag))], Some("Special")),
        ("Delimiter", vec![Part::All(r_syntax(X::Punctuation))], None),
        (
            "SpecialComment",
            vec![Part::All(r_syntax(X::DocComment))],
            Some("Comment"),
        ),
        ("Debug", vec![], Some("Special")),
        // Messages
        (
            "ErrorMsg",
            vec![Part::Fg(r_diag(DK::Error, DP::Text))],
            None,
        ),
        (
            "WarningMsg",
            vec![Part::Fg(r_diag(DK::Warning, DP::Text))],
            None,
        ),
        // Diagnostics: text, sign, underline, virtual text per kind
        (
            "DiagnosticError",
            vec![Part::Fg(r_diag(DK::Error, DP::Text))],
            None,
        ),
        (
            "DiagnosticWarn",
            vec![Part::Fg(r_diag(DK::Warning, DP::Text))],
            None,
        ),
        (
            "DiagnosticInfo",
            vec![Part::Fg(r_diag(DK::Info, DP::Text))],
            None,
        ),
        (
            "DiagnosticHint",
            vec![Part::Fg(r_diag(DK::Hint, DP::Text))],
            None,
        ),
        (
            "DiagnosticSignError",
            vec![Part::Fg(r_diag(DK::Error, DP::Sign))],
            Some("DiagnosticError"),
        ),
        (
            "DiagnosticSignWarn",
            vec![Part::Fg(r_diag(DK::Warning, DP::Sign))],
            Some("DiagnosticWarn"),
        ),
        (
            "DiagnosticSignInfo",
            vec![Part::Fg(r_diag(DK::Info, DP::Sign))],
            Some("DiagnosticInfo"),
        ),
        (
            "DiagnosticSignHint",
            vec![Part::Fg(r_diag(DK::Hint, DP::Sign))],
            Some("DiagnosticHint"),
        ),
        (
            "DiagnosticVirtualTextError",
            vec![Part::Fg(r_diag(DK::Error, DP::VirtualText))],
            Some("DiagnosticError"),
        ),
        (
            "DiagnosticVirtualTextWarn",
            vec![Part::Fg(r_diag(DK::Warning, DP::VirtualText))],
            Some("DiagnosticWarn"),
        ),
        (
            "DiagnosticVirtualTextInfo",
            vec![Part::Fg(r_diag(DK::Info, DP::VirtualText))],
            Some("DiagnosticInfo"),
        ),
        (
            "DiagnosticVirtualTextHint",
            vec![Part::Fg(r_diag(DK::Hint, DP::VirtualText))],
            Some("DiagnosticHint"),
        ),
        // Diff
        (
            "DiffAdd",
            vec![Part::Bg(r_diff(XK::Added, XP::LineBg))],
            None,
        ),
        (
            "DiffChange",
            vec![Part::Bg(r_diff(XK::Changed, XP::LineBg))],
            None,
        ),
        (
            "DiffDelete",
            vec![Part::Bg(r_diff(XK::Removed, XP::LineBg))],
            None,
        ),
        ("DiffText", vec![], Some("DiffChange")),
        // Semantic-ish roles with no standard vim group: direct when the
        // IR defines them, aliased to the nearest standard group otherwise.
        (
            "@parameter",
            vec![Part::All(r_syntax(X::Parameter))],
            Some("Identifier"),
        ),
        (
            "@property",
            vec![Part::All(r_syntax(X::Property))],
            Some("Identifier"),
        ),
        (
            "@variable.builtin",
            vec![Part::All(r_syntax(X::VariableBuiltin))],
            Some("Identifier"),
        ),
        (
            "@namespace",
            vec![Part::All(r_syntax(X::Namespace))],
            Some("Identifier"),
        ),
        (
            "@attribute",
            vec![Part::All(r_syntax(X::Attribute))],
            Some("Identifier"),
        ),
        (
            "@tag.attribute",
            vec![Part::All(r_syntax(X::Attribute))],
            Some("Identifier"),
        ),
    ];
    entries
}

/// Ecosystem layer: (group, link target). Linked groups inherit whatever
/// the target resolves to, so user overrides keep working.
fn link_definitions() -> Vec<(&'static str, &'static str)> {
    vec![
        // Tree-sitter captures
        ("@comment", "Comment"),
        ("@comment.documentation", "SpecialComment"),
        ("@comment.error", "DiagnosticError"),
        ("@string", "String"),
        ("@string.escape", "Special"),
        ("@string.special", "Special"),
        ("@character", "Character"),
        ("@character.special", "Special"),
        ("@number", "Number"),
        ("@boolean", "Boolean"),
        ("@float", "Float"),
        ("@function", "Function"),
        ("@function.call", "Function"),
        ("@function.builtin", "Function"),
        ("@method", "Function"),
        ("@method.call", "Function"),
        ("@constructor", "Delimiter"),
        ("@keyword", "Keyword"),
        ("@keyword.function", "Keyword"),
        ("@keyword.operator", "Operator"),
        ("@keyword.return", "Keyword"),
        ("@conditional", "Keyword"),
        ("@repeat", "Keyword"),
        ("@label", "Label"),
        ("@include", "Include"),
        ("@exception", "Keyword"),
        ("@debug", "Debug"),
        ("@operator", "Operator"),
        ("@punctuation.bracket", "Delimiter"),
        ("@punctuation.delimiter", "Delimiter"),
        ("@punctuation.special", "Special"),
        ("@variable", "Identifier"),
        ("@constant", "Constant"),
        ("@constant.builtin", "Constant"),
        ("@constant.macro", "Macro"),
        ("@type", "Type"),
        ("@type.builtin", "Type"),
        ("@type.definition", "Type"),
        ("@type.qualifier", "Type"),
        ("@annotation", "Comment"),
        ("@tag", "Tag"),
        ("@tag.delimiter", "Delimiter"),
        ("@macro", "Macro"),
        // LSP semantic tokens
        ("@lsp.type.class", "Type"),
        ("@lsp.type.enum", "Type"),
        ("@lsp.type.enumMember", "Constant"),
        ("@lsp.type.struct", "Type"),
        ("@lsp.type.interface", "Type"),
        ("@lsp.type.typeParameter", "Type"),
        ("@lsp.type.function", "Function"),
        ("@lsp.type.method", "Function"),
        ("@lsp.type.macro", "Macro"),
        ("@lsp.type.namespace", "Identifier"),
        ("@lsp.type.parameter", "@parameter"),
        ("@lsp.type.property", "@property"),
        ("@lsp.type.variable", "Identifier"),
        ("@lsp.type.decorator", "Function"),
        ("@lsp.type.event", "Special"),
        ("@lsp.type.modifier", "Keyword"),
    ]
}

/// Groups whose definitions need post-processing beyond the generic path.
fn underline_attrs(style: &Style) -> Vec<(&'static str, bool)> {
    let mut attrs = match style.underline {
        Some(UnderlineStyle::Single) => vec![("underline", true)],
        Some(UnderlineStyle::Double) => vec![("underdouble", true)],
        Some(UnderlineStyle::Curl) => vec![("undercurl", true)],
        Some(UnderlineStyle::Dot) => vec![("underdotted", true)],
        Some(UnderlineStyle::Dash) => vec![("underdashed", true)],
        Some(UnderlineStyle::Strikethrough) => vec![("strikethrough", true)],
        None if style.states.get(&crate::ir::Property::Underline)
            == Some(&crate::ir::PropertyState::Clear) =>
        {
            vec![
                ("underline", false),
                ("undercurl", false),
                ("underdouble", false),
                ("underdotted", false),
                ("underdashed", false),
            ]
        }
        None => vec![],
    };
    if let Some(strike) = style.strikethrough {
        attrs.push(("strikethrough", strike));
    } else if style.states.get(&crate::ir::Property::Strikethrough)
        == Some(&crate::ir::PropertyState::Clear)
    {
        attrs.push(("strikethrough", false));
    }
    attrs
}

fn lua_color(c: Srgb) -> String {
    c.to_hex()
}

fn sanitize_name(name: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in name.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            prev_dash = false;
            ch.to_ascii_lowercase()
        } else {
            if !prev_dash && !out.is_empty() {
                prev_dash = true;
                '-'
            } else {
                continue;
            }
        };
        out.push(mapped);
    }
    out.trim_matches('-').to_string()
}

/// Emit a complete Neovim colorscheme Lua file plus an honest report.
pub fn emit_nvim(theme: &Theme) -> Result<NvimEmit, EmitError> {
    theme.validate().map_err(EmitError::InvalidTheme)?;
    let mut lowered = theme.clone();
    let mut property_losses = vec![];
    let base = theme.base_bg().unwrap_or(Srgb::BLACK);
    for (role, style) in &mut lowered.styles {
        if let Some(resolved) = crate::validate::resolved_style(theme, role) {
            *style = resolved;
        }
        for (property, alpha) in [
            (crate::ir::Property::Fg, style.fg_opacity),
            (crate::ir::Property::Bg, style.bg_opacity),
            (crate::ir::Property::Special, style.special_opacity),
        ] {
            if let Some(alpha) = alpha.filter(|a| *a < 1.0) {
                let color = match property {
                    crate::ir::Property::Fg => &mut style.fg,
                    crate::ir::Property::Bg => &mut style.bg,
                    _ => &mut style.special,
                };
                if let Some(c) = color {
                    *c = crate::color::composite_over(*c, alpha, base);
                }
                property_losses.push((
                    role.to_string(),
                    format!("{property:?}"),
                    "alpha flattened against base; context-dependent backgrounds not preserved"
                        .into(),
                ));
            }
        }
        for (property, state) in &style.states {
            if *property == crate::ir::Property::Underline
                && *state == crate::ir::PropertyState::Clear
            {
                continue;
            }
            property_losses.push((
                role.to_string(),
                format!("{property:?}"),
                format!("explicit {state:?} lowering requires target defaults; emitted omission"),
            ));
        }
    }
    let semantic = super::semantic_nvim::emit(theme).map_err(EmitError::InvalidTheme)?;
    property_losses.extend(semantic.losses);
    let theme = &lowered;
    if theme.base_bg().is_none() {
        return Err(EmitError::MissingBase(theme.meta.name.clone()));
    }
    if theme.fg(&Role::Text(TextRole::Primary)).is_none() {
        return Err(EmitError::MissingPrimary(theme.meta.name.clone()));
    }

    let mut report = EmitReport {
        losses: property_losses,
        ..EmitReport::default()
    };
    let mut consumed: HashSet<String> = HashSet::new();
    let mut lua = String::new();

    let colors_name = format!("hexwitch-{}", sanitize_name(&theme.meta.name));
    let background = match theme.meta.appearance {
        Appearance::Dark => "dark",
        Appearance::Light => "light",
    };
    // Metadata is data, not Lua. Keep it on a single comment line so an
    // imported IR source containing newlines cannot inject executable code.
    let source: String = theme
        .meta
        .source
        .as_deref()
        .unwrap_or("unknown")
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();

    let _ = write!(
        lua,
        "-- Generated by {by}. Do not edit by hand.\n\
         -- Source: {source}\n\n\
         if vim.g.colors_name then\n  vim.cmd(\"highlight clear\")\nend\n\
         if vim.fn.exists(\"syntax_on\") then\n  vim.cmd(\"syntax reset\")\nend\n\
         vim.o.background = \"{background}\"\n\
         vim.g.colors_name = \"{colors_name}\"\n\n",
        by = crate::emit::EMITTED_BY,
        source = source,
        background = background,
        colors_name = colors_name,
    );

    // ---- base definitions ----
    for (group, parts, fallback) in base_definitions() {
        let mut opts: Vec<String> = Vec::new();
        for part in &parts {
            match part {
                Part::Fg(role) => {
                    if let Some(style) = theme.get(role) {
                        consumed.insert(role.to_string());
                        if let Some(fg) = style.fg {
                            opts.push(format!("fg = \"{}\"", lua_color(fg)));
                        }
                        if let Some(sp) = style.special {
                            opts.push(format!("sp = \"{}\"", lua_color(sp)));
                        }
                        for (k, v) in underline_attrs(style) {
                            opts.push(format!("{k} = {v}"));
                        }
                        if let Some(bold) = style.bold {
                            opts.push(format!("bold = {bold}"));
                        }
                        if let Some(italic) = style.italic {
                            opts.push(format!("italic = {italic}"));
                        }
                    }
                }
                Part::Bg(role) => {
                    if let Some(style) = theme.get(role) {
                        consumed.insert(role.to_string());
                        if let Some(bg) = style.bg {
                            opts.push(format!("bg = \"{}\"", lua_color(bg)));
                        }
                        if let Some(blend) = style.blend {
                            opts.push(format!("blend = {blend}"));
                        }
                    }
                }
                Part::All(role) => {
                    if let Some(style) = theme.get(role) {
                        consumed.insert(role.to_string());
                        if let Some(fg) = style.fg {
                            opts.push(format!("fg = \"{}\"", lua_color(fg)));
                        }
                        if let Some(bg) = style.bg {
                            opts.push(format!("bg = \"{}\"", lua_color(bg)));
                        }
                        if let Some(sp) = style.special {
                            opts.push(format!("sp = \"{}\"", lua_color(sp)));
                        }
                        for (k, v) in underline_attrs(style) {
                            opts.push(format!("{k} = {v}"));
                        }
                        if let Some(bold) = style.bold {
                            opts.push(format!("bold = {bold}"));
                        }
                        if let Some(italic) = style.italic {
                            opts.push(format!("italic = {italic}"));
                        }
                        if let Some(blend) = style.blend {
                            opts.push(format!("blend = {blend}"));
                        }
                    }
                }
            }
        }

        if !opts.is_empty() {
            let _ = writeln!(
                lua,
                "vim.api.nvim_set_hl(0, \"{}\", {{ {} }})",
                group,
                opts.join(", ")
            );
            report.defined.push(group.to_string());
        } else if let Some(target) = fallback {
            let _ = writeln!(
                lua,
                "vim.api.nvim_set_hl(0, \"{}\", {{ link = \"{}\" }})",
                group, target
            );
            report.linked.push(group.to_string());
        } else {
            let missing = parts
                .first()
                .map(|p| match p {
                    Part::Fg(r) | Part::Bg(r) | Part::All(r) => r.to_string(),
                })
                .unwrap_or_default();
            report.skipped.push((group.to_string(), missing));
        }
    }

    // ---- diagnostic underline groups ----
    // These carry their color in `sp` (the decoration color), plus a fixed
    // undercurl decoration; critical signals must not rely on color alone.
    let underline_groups = [
        ("DiagnosticUnderlineError", DiagnosticKind::Error),
        ("DiagnosticUnderlineWarn", DiagnosticKind::Warning),
        ("DiagnosticUnderlineInfo", DiagnosticKind::Info),
        ("DiagnosticUnderlineHint", DiagnosticKind::Hint),
    ];
    for (group, kind) in underline_groups {
        let role = Role::Diagnostic {
            kind,
            part: DiagnosticPart::Text,
        };
        if let Some(color) = theme.fg(&role) {
            consumed.insert(role.to_string());
            let _ = writeln!(
                lua,
                "vim.api.nvim_set_hl(0, \"{group}\", {{ sp = \"{}\", undercurl = true }})",
                lua_color(color)
            );
            report.defined.push(group.to_string());
        } else {
            report.skipped.push((group.to_string(), role.to_string()));
        }
    }

    // ---- links ----
    for (group, target) in link_definitions() {
        let _ = writeln!(
            lua,
            "vim.api.nvim_set_hl(0, \"{}\", {{ link = \"{}\" }})",
            group, target
        );
        report.linked.push(group.to_string());
    }

    // Explicit semantic definitions follow generic LSP links so they are not overwritten.
    lua.push_str(&semantic.lua);
    report.defined.extend(semantic.defined);

    // ---- terminal palette ----
    for slot in 0u8..16 {
        if let Some(style) = theme.get(&Role::Terminal(slot)) {
            if let Some(fg) = style.fg {
                consumed.insert(format!("terminal.{slot}"));
                let _ = writeln!(lua, "vim.g.terminal_color_{slot} = \"{}\"", lua_color(fg));
            }
        }
    }

    // ---- unconsumed roles ----
    for role in theme.styles.keys() {
        if !consumed.contains(&role.to_string()) {
            report.unconsumed_roles.push(role.to_string());
        }
    }
    if theme
        .get(&"other.terminal-default".parse().unwrap())
        .is_some()
    {
        report.losses.push((
            "other.terminal-default".into(),
            "fg/bg".into(),
            "Neovim built-in terminal defaults follow Normal, not an independent terminal surface"
                .into(),
        ));
    }

    let _ = writeln!(lua, "\n-- vim: sw=2 ts=2 et");

    Ok(NvimEmit { lua, report })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::style::Provenance;

    fn gruvbox_ish() -> Theme {
        let mut t = Theme::new("Gruvbox-ish Dark", Appearance::Dark);
        t.set(
            Role::Surface(Surface::Base),
            Style::bg(Srgb::from_hex("#282828").unwrap()),
        );
        t.set(
            Role::Text(TextRole::Primary),
            Style::fg(Srgb::from_hex("#ebdbb2").unwrap()),
        );
        t.set(
            Role::Text(TextRole::Secondary),
            Style::fg(Srgb::from_hex("#bdae93").unwrap()),
        );
        t.set(
            Role::Syntax(SyntaxRole::Comment),
            Style::fg(Srgb::from_hex("#928374").unwrap()),
        );
        t.set(
            Role::Syntax(SyntaxRole::String),
            Style::fg(Srgb::from_hex("#b8bb26").unwrap()),
        );
        t.set(
            Role::Syntax(SyntaxRole::Keyword),
            Style::fg(Srgb::from_hex("#fb4934").unwrap()),
        );
        t.set(
            Role::Syntax(SyntaxRole::Function),
            Style::fg(Srgb::from_hex("#b8bb26").unwrap()).with_provenance(Provenance::Locked),
        );
        t.set(
            Role::Terminal(1),
            Style::fg(Srgb::from_hex("#cc241d").unwrap()),
        );
        t
    }

    #[test]
    fn emits_loadable_lua_with_expected_groups() {
        let out = emit_nvim(&gruvbox_ish()).unwrap();
        assert!(out.lua.contains("vim.o.background = \"dark\""));
        assert!(out
            .lua
            .contains("vim.g.colors_name = \"hexwitch-gruvbox-ish-dark\""));
        assert!(out.lua.contains(
            "vim.api.nvim_set_hl(0, \"Normal\", { fg = \"#ebdbb2\", bg = \"#282828\" })"
        ));
        assert!(out.lua.contains("fg = \"#928374\""));
        assert!(out.lua.contains("vim.g.terminal_color_1 = \"#cc241d\""));
        // Captures link, not duplicate.
        assert!(out
            .lua
            .contains("vim.api.nvim_set_hl(0, \"@comment\", { link = \"Comment\" })"));
        assert!(out.lua.contains("link = \"@parameter\""));
    }

    #[test]
    fn missing_roles_fall_back_to_links_or_are_reported() {
        let out = emit_nvim(&gruvbox_ish()).unwrap();
        // No boolean role -> falls back to Constant.
        assert!(out.lua.contains("{ link = \"Constant\" }"));
        // No interaction roles -> Visual/CursorLine skipped (and reported
        // with the missing role name).
        assert!(out.report.skipped.iter().any(|(g, _)| g == "Visual"));
        assert!(out.report.skipped.iter().any(|(g, _)| g == "LineNr"));
        assert!(out
            .report
            .skipped
            .iter()
            .any(|(g, r)| g == "Visual" && r == "interaction.selection"));
    }

    #[test]
    fn source_metadata_cannot_escape_the_comment() {
        let mut theme = gruvbox_ish();
        theme.meta.source = Some("reference\nerror('injected')\rnext".into());
        let emitted = emit_nvim(&theme).unwrap();
        assert!(emitted
            .lua
            .contains("-- Source: reference error('injected') next"));
        assert!(!emitted.lua.contains("\nerror('injected')"));
    }

    #[test]
    fn requires_base_and_primary() {
        let mut t = Theme::new("empty", Appearance::Dark);
        assert!(matches!(emit_nvim(&t), Err(EmitError::MissingBase(_))));
        t.set(Role::Surface(Surface::Base), Style::bg(Srgb::BLACK));
        assert!(matches!(emit_nvim(&t), Err(EmitError::MissingPrimary(_))));
    }

    #[test]
    fn italic_and_underline_survive() {
        let mut t = gruvbox_ish();
        let mut comment = t.get(&Role::Syntax(SyntaxRole::Comment)).cloned().unwrap();
        comment.italic = Some(true);
        comment.underline = Some(UnderlineStyle::Curl);
        t.set(Role::Syntax(SyntaxRole::Comment), comment);
        let out = emit_nvim(&t).unwrap();
        assert!(out.lua.contains("italic = true"));
        assert!(out.lua.contains("undercurl = true"));
    }
}
