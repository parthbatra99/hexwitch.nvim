//! Intent v1: explicit property targets and persistent constraints.
use crate::color::{contrast_ratio, gamut_map, Oklch, Srgb};
use crate::ir::{Origin, Property, PropertyState, Role, SourceInfo, Theme, UnderlineStyle};
use crate::protocol::Outcome;
use crate::solver::{solve_contrast, Direction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntentSpec {
    pub version: u32,
    pub actions: Vec<Action>,
    #[serde(default)]
    pub requirements: Vec<Constraint>,
    #[serde(default)]
    pub preferences: Vec<Preference>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub roles: Vec<String>,
    pub property: Property,
    /// Include all existing semantic overrides as well as the lexical role.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub include_semantic: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Request,
    #[default]
    Persistent,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Lock {
        target: Target,
        #[serde(default)]
        scope: Scope,
    },
    Unlock {
        target: Target,
    },
    SetColor {
        target: Target,
        color: Srgb,
    },
    SetNumber {
        target: Target,
        value: f64,
    },
    SetBool {
        target: Target,
        value: bool,
    },
    SetUnderline {
        target: Target,
        value: UnderlineStyle,
    },
    State {
        target: Target,
        state: PropertyState,
    },
    Lightness {
        target: Target,
        delta: f64,
    },
    Chroma {
        target: Target,
        delta: f64,
    },
    Hue {
        target: Target,
        degrees: f64,
    },
    Contrast {
        target: Target,
        backgrounds: Vec<crate::backdrop::Backdrop>,
        min: f64,
        #[serde(default)]
        direction: Direction,
    },
}
impl Action {
    pub fn target(&self) -> &Target {
        match self {
            Self::Lock { target, .. }
            | Self::Unlock { target }
            | Self::SetColor { target, .. }
            | Self::SetNumber { target, .. }
            | Self::SetBool { target, .. }
            | Self::SetUnderline { target, .. }
            | Self::State { target, .. }
            | Self::Lightness { target, .. }
            | Self::Chroma { target, .. }
            | Self::Hue { target, .. }
            | Self::Contrast { target, .. } => target,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    pub role: Role,
    pub property: Property,
    pub backgrounds: Vec<crate::backdrop::Backdrop>,
    pub min: f64,
    #[serde(default)]
    pub persistent: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub include_semantic: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preference {
    pub target: Target,
    pub weight: f64,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct PropertyOutcome {
    pub role: Role,
    pub property: Property,
    pub outcome: Outcome,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_rule: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_context: Option<crate::semantic::SemanticContext>,
}
#[derive(Debug, Clone, Serialize)]
pub struct IntentReport {
    pub outcome: Outcome,
    pub properties: Vec<PropertyOutcome>,
    pub requirements: Vec<PropertyOutcome>,
    pub preferences: Vec<Preference>,
    /// Required constraints failed; proposed property changes were not retained.
    pub rolled_back: bool,
}

fn roles(theme: &Theme, target: &Target) -> Result<Vec<Role>, String> {
    if target.roles.is_empty() {
        return Err("empty role collection".into());
    }
    let mut roles = std::collections::BTreeSet::new();
    for selector in &target.roles {
        if !crate::selector::valid_selector(selector) {
            return Err(format!("invalid selector {selector}"));
        }
        let found: Vec<_> = theme
            .styles
            .keys()
            .filter(|r| crate::selector::matches(selector, r))
            .cloned()
            .collect();
        if found.is_empty() {
            return Err(format!("missing target {selector}"));
        }
        roles.extend(found);
    }
    Ok(roles.into_iter().collect())
}
fn color(style: &crate::ir::Style, property: Property) -> Option<Srgb> {
    match property {
        Property::Fg => style.fg,
        Property::Bg => style.bg,
        Property::Special => style.special,
        _ => None,
    }
}
fn opacity(style: &crate::ir::Style, property: Property) -> f64 {
    match property {
        Property::Fg => style.fg_opacity,
        Property::Bg => style.bg_opacity,
        Property::Special => style.special_opacity,
        _ => None,
    }
    .unwrap_or(1.0)
}
fn set_color(style: &mut crate::ir::Style, property: Property, value: Srgb) {
    match property {
        Property::Fg => style.fg = Some(value),
        Property::Bg => style.bg = Some(value),
        Property::Special => style.special = Some(value),
        _ => unreachable!(),
    }
}
pub(crate) fn validate_action(action: &Action) -> Result<(), String> {
    let p = action.target().property;
    let is_color = matches!(p, Property::Fg | Property::Bg | Property::Special);
    let ok = match action {
        Action::SetColor { .. } => is_color,
        Action::SetNumber { value, .. } => {
            value.is_finite()
                && match p {
                    Property::FgOpacity | Property::BgOpacity | Property::SpecialOpacity => {
                        (0.0..=1.0).contains(value)
                    }
                    Property::Blend => (0.0..=100.0).contains(value) && value.fract() == 0.0,
                    _ => false,
                }
        }
        Action::SetBool { .. } => matches!(
            p,
            Property::Bold | Property::Italic | Property::Strikethrough
        ),
        Action::SetUnderline { .. } => p == Property::Underline,
        Action::Lightness { delta, .. } | Action::Chroma { delta, .. } => {
            is_color && delta.is_finite() && (-1.0..=1.0).contains(delta)
        }
        Action::Hue { degrees, .. } => {
            is_color && degrees.is_finite() && (-180.0..=180.0).contains(degrees)
        }
        Action::Contrast {
            min, backgrounds, ..
        } => {
            is_color
                && min.is_finite()
                && (1.0..=21.0).contains(min)
                && !backgrounds.is_empty()
                && backgrounds.len() <= 64
        }
        _ => true,
    };
    if ok {
        Ok(())
    } else {
        Err(format!("invalid operation or value for {p:?}"))
    }
}

pub fn revise_intent(theme: &Theme, spec: &IntentSpec) -> Result<(Theme, IntentReport), String> {
    theme.validate()?;
    if spec.version != 1 {
        return Err("unsupported intent version".into());
    }
    if spec.actions.len() > 1024 || spec.requirements.len() > 1024 {
        return Err("intent limit exceeded".into());
    }
    for action in &spec.actions {
        roles(theme, action.target())?;
        validate_action(action)?;
        if let Action::Contrast { backgrounds, .. } = action {
            for b in backgrounds {
                b.validate(theme)?;
            }
        }
    }
    for preference in &spec.preferences {
        roles(theme, &preference.target)?;
        if !preference.weight.is_finite() || !(0.0..=1.0).contains(&preference.weight) {
            return Err("invalid preference weight".into());
        }
    }
    let mut constraints: Vec<Constraint> = theme
        .extensions
        .get("hexwitch.constraints.v1")
        .map(|v| serde_json::from_value(v.clone()))
        .transpose()
        .map_err(|e| format!("invalid stored constraints: {e}"))?
        .unwrap_or_default();
    for requirement in &spec.requirements {
        if !constraints.contains(requirement) {
            constraints.push(requirement.clone());
        }
    }
    if constraints.len() > 1024 || spec.preferences.len() > 1024 {
        return Err("persistent constraint or preference count limit exceeded".into());
    }
    for requirement in &constraints {
        if !theme.styles.contains_key(&requirement.role)
            || !matches!(
                requirement.property,
                Property::Fg | Property::Bg | Property::Special
            )
            || requirement.backgrounds.is_empty()
            || requirement.backgrounds.len() > 64
            || !requirement.min.is_finite()
            || !(1.0..=21.0).contains(&requirement.min)
        {
            return Err("invalid requirement".into());
        }
        for b in &requirement.backgrounds {
            b.validate(theme)?;
        }
    }
    let mut revised = theme.clone();
    let mut report = IntentReport {
        outcome: Outcome::Unchanged,
        properties: vec![],
        requirements: vec![],
        preferences: spec.preferences.clone(),
        rolled_back: false,
    };
    let mut local_locks = std::collections::BTreeSet::new();
    for action in &spec.actions {
        let target = action.target();
        let contrast_backgrounds = match action {
            Action::Contrast { backgrounds, .. } => {
                Some(crate::backdrop::resolve_all(&revised, backgrounds))
            }
            _ => None,
        };
        let mut targets = vec![];
        for role in roles(&revised, target)? {
            targets.push((role.clone(), None));
            if target.include_semantic {
                targets.extend(
                    revised
                        .modifiers
                        .iter()
                        .enumerate()
                        .filter(|(_, rule)| rule.role == role)
                        .map(|(index, _)| (role.clone(), Some(index))),
                );
            }
        }
        for (role, semantic_rule) in targets {
            let p = target.property;
            // Semantic patches often omit fg/opacity and inherit the lexical
            // treatment. Solve that effective color without filling other slots.
            let inherited = crate::validate::resolved_style(&revised, &role);
            let effective_opacity = match semantic_rule {
                Some(index) => {
                    let patch = &revised.modifiers[index].style;
                    let own = match p {
                        Property::Fg => patch.fg_opacity,
                        Property::Bg => patch.bg_opacity,
                        Property::Special => patch.special_opacity,
                        _ => None,
                    };
                    own.unwrap_or_else(|| inherited.as_ref().map(|s| opacity(s, p)).unwrap_or(1.0))
                }
                None => inherited.as_ref().map(|s| opacity(s, p)).unwrap_or(1.0),
            };
            let style = match semantic_rule {
                Some(index) => &mut revised.modifiers[index].style,
                None => revised.styles.get_mut(&role).unwrap(),
            };
            let before = style.clone();
            let mut outcome = Outcome::Satisfied;
            let mut reason = String::new();
            match action {
                Action::Lock { scope, .. } => match scope {
                    Scope::Request => {
                        local_locks.insert((role.clone(), semantic_rule, p));
                    }
                    Scope::Persistent => {
                        style.locked.insert(p);
                    }
                },
                Action::Unlock { .. } => {
                    if style.provenance == crate::ir::Provenance::Locked {
                        style.locked.extend(Property::ALL);
                        style.provenance = crate::ir::Provenance::Ingested;
                    }
                    style.locked.remove(&p);
                    local_locks.remove(&(role.clone(), semantic_rule, p));
                }
                _ if style.is_locked(p)
                    || local_locks.contains(&(role.clone(), semantic_rule, p)) =>
                {
                    outcome = Outcome::Refused;
                    reason = "property is locked".into();
                }
                Action::State { state, .. } => {
                    style.clear_value(p);
                    style.states.insert(p, *state);
                }
                Action::SetNumber { value, .. } => {
                    match p {
                        Property::FgOpacity => style.fg_opacity = Some(*value),
                        Property::BgOpacity => style.bg_opacity = Some(*value),
                        Property::SpecialOpacity => style.special_opacity = Some(*value),
                        Property::Blend => style.blend = Some(*value as u8),
                        _ => unreachable!(),
                    };
                    style.states.remove(&p);
                }
                Action::SetBool { value, .. } => {
                    match p {
                        Property::Bold => style.bold = Some(*value),
                        Property::Italic => style.italic = Some(*value),
                        Property::Strikethrough => style.strikethrough = Some(*value),
                        _ => unreachable!(),
                    }
                    style.states.remove(&p);
                }
                Action::SetUnderline { value, .. } => {
                    style.underline = Some(*value);
                    style.states.remove(&p);
                }
                _ => {
                    let new =
                        match action {
                            Action::SetColor { color, .. } => Ok(*color),
                            _ => match color(style, p).or_else(|| {
                                if !style.states.contains_key(&p)
                                    || style.states.get(&p) == Some(&PropertyState::Inherit)
                                {
                                    inherited.as_ref().and_then(|s| color(s, p))
                                } else {
                                    None
                                }
                            }) {
                                None => Err("target property has no concrete color".into()),
                                Some(current) => {
                                    let ok = current.to_oklch();
                                    match action {
                                        Action::Lightness { delta, .. } => Ok(gamut_map(
                                            Oklch::new((ok.l + delta).clamp(0.0, 1.0), ok.c, ok.h),
                                        )),
                                        Action::Chroma { delta, .. } => Ok(gamut_map(Oklch::new(
                                            ok.l,
                                            (ok.c + delta).max(0.0),
                                            ok.h,
                                        ))),
                                        Action::Hue { degrees, .. } => {
                                            Ok(gamut_map(Oklch::new(ok.l, ok.c, ok.h + degrees)))
                                        }
                                        Action::Contrast { min, direction, .. } => {
                                            match contrast_backgrounds.as_ref().unwrap() {
                                                Ok(backgrounds) => solve_contrast(
                                                    current,
                                                    backgrounds,
                                                    *min,
                                                    *direction,
                                                    effective_opacity,
                                                )
                                                .map(|r| r.color)
                                                .map_err(|e| e.to_string()),
                                                Err(reason) => Err(reason.clone()),
                                            }
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                            },
                        };
                    match new {
                        Ok(c) => {
                            set_color(style, p, c);
                            style.states.remove(&p);
                        }
                        Err(e) => {
                            outcome = Outcome::Infeasible;
                            reason = e;
                        }
                    }
                }
            }
            if *style == before && outcome == Outcome::Satisfied {
                outcome = Outcome::Unchanged;
            }
            if *style != before && !matches!(action, Action::Lock { .. } | Action::Unlock { .. }) {
                style.source = Some(SourceInfo {
                    origin: Origin::Generated,
                    reference: Some(theme.meta.name.clone()),
                });
            }
            report.properties.push(PropertyOutcome {
                role,
                property: p,
                outcome,
                reason,
                semantic_rule,
                semantic_context: None,
            });
        }
    }
    for requirement in &constraints {
        let backgrounds = crate::backdrop::resolve_all(&revised, &requirement.backgrounds);
        let base = crate::validate::resolved_style(&revised, &requirement.role);
        let mut treatments = vec![(None, base.clone())];
        if requirement.include_semantic {
            match crate::semantic_constraints::treatments(
                &revised,
                &requirement.role,
                &base.clone().unwrap_or_default(),
            ) {
                Ok(contexts) => treatments.extend(
                    contexts
                        .into_iter()
                        .map(|(context, style)| (Some(context), Some(style))),
                ),
                Err(reason) => report.requirements.push(PropertyOutcome {
                    role: requirement.role.clone(),
                    property: requirement.property,
                    outcome: Outcome::Infeasible,
                    reason,
                    semantic_rule: None,
                    semantic_context: None,
                }),
            }
        }
        for (semantic_context, treatment) in treatments {
            let achieved = treatment.as_ref().and_then(|s| {
                color(s, requirement.property)
                    .zip(backgrounds.as_ref().ok())
                    .map(|(c, backgrounds)| {
                        backgrounds
                            .iter()
                            .map(|b| {
                                contrast_ratio(
                                    crate::color::composite_over(
                                        c,
                                        opacity(s, requirement.property),
                                        *b,
                                    ),
                                    *b,
                                )
                            })
                            .fold(21.0, f64::min)
                    })
            });
            let satisfied = achieved.is_some_and(|r| r >= requirement.min);
            report.requirements.push(PropertyOutcome {
                role: requirement.role.clone(),
                property: requirement.property,
                outcome: if satisfied {
                    Outcome::Satisfied
                } else {
                    Outcome::Infeasible
                },
                reason: format!(
                    "final contrast {:?}; required {}; {}",
                    achieved,
                    requirement.min,
                    backgrounds
                        .as_ref()
                        .err()
                        .map(String::as_str)
                        .unwrap_or("backgrounds resolved")
                ),
                semantic_rule: None,
                semantic_context,
            });
        }
    }
    if report
        .requirements
        .iter()
        .any(|r| r.outcome == Outcome::Infeasible)
    {
        // Required constraints are transactional: return the original, never an invalid candidate.
        report.outcome = Outcome::Infeasible;
        report.rolled_back = true;
        return Ok((theme.clone(), report));
    }
    let persistent: Vec<_> = constraints.into_iter().filter(|r| r.persistent).collect();
    if !persistent.is_empty() {
        revised.extensions.insert(
            "hexwitch.constraints.v1".into(),
            serde_json::to_value(persistent).map_err(|e| e.to_string())?,
        );
    }
    let issues = report
        .properties
        .iter()
        .any(|r| matches!(r.outcome, Outcome::Refused | Outcome::Infeasible));
    let changed = revised != *theme;
    report.outcome = if issues && changed {
        Outcome::Partial
    } else if issues {
        if report
            .properties
            .iter()
            .any(|r| r.outcome == Outcome::Refused)
        {
            Outcome::Refused
        } else {
            Outcome::Infeasible
        }
    } else if changed {
        Outcome::Satisfied
    } else {
        Outcome::Unchanged
    };
    revised.validate()?;
    Ok((revised, report))
}
