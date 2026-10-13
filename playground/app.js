'use strict';
const $ = id => document.getElementById(id);
const SVG_NS = 'http://www.w3.org/2000/svg';

function el(tag, attrs = {}, ...children) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) {
    if (value == null || value === false) continue;
    if (key === 'text') node.textContent = value;
    else if (key === 'class') node.className = value;
    else if (key === 'style') node.style.cssText = value;
    else if (key.startsWith('on') && typeof value === 'function') node.addEventListener(key.slice(2).toLowerCase(), value);
    else if (key === 'value') node.value = value;
    else node.setAttribute(key, value === true ? '' : value);
  }
  for (const child of children.flat()) if (child != null && child !== false) node.append(child);
  return node;
}

const ICONS = {
  check: '<path d="M5 12.5l4.5 4.5L19 7.5"/>',
  arrow: '<path d="M5 12h14M13 6l6 6-6 6"/>',
  close: '<path d="M6 6l12 12M18 6L6 18"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  star: '<path d="M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9z"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="M20 20l-3.5-3.5"/>',
  hex: '<path d="M12 2.5l8.4 4.85v9.3L12 21.5l-8.4-4.85v-9.3z"/>',
  alert: '<circle cx="12" cy="12" r="9"/><path d="M12 8v5M12 16.5v.5"/>',
  question: '<circle cx="12" cy="12" r="9"/><path d="M9.5 9.5a2.5 2.5 0 114 2c-.9.6-1.5 1.1-1.5 2.2M12 17v.5"/>',
  offline: '<path d="M3 3l18 18M8.5 8.6A5 5 0 0012 17h6a4 4 0 001.6-7.7M6.3 6.4A6 6 0 0115.7 7"/>',
  shield: '<path d="M12 3l8 3v6c0 4.5-3.4 8.3-8 9-4.6-.7-8-4.5-8-9V6z"/>',
  monitor: '<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/>',
  link: '<path d="M10 14a4 4 0 005.7 0l3-3a4 4 0 00-5.7-5.7l-1 1M14 10a4 4 0 00-5.7 0l-3 3a4 4 0 005.7 5.7l1-1"/>',
  chevron: '<path d="M9 6l6 6-6 6"/>',
  palette: '<circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>',
  stop: '<rect x="7" y="7" width="10" height="10" rx="2"/>',
  send: '<path d="M12 19V5M5.5 11.5L12 5l6.5 6.5"/>',
  retry: '<path d="M4 12a8 8 0 0113.7-5.7L20 8.5M20 4v4.5h-4.5"/>',
};
function icon(name, className) {
  const node = document.createElementNS(SVG_NS, 'svg');
  node.setAttribute('viewBox', '0 0 24 24');
  node.setAttribute('aria-hidden', 'true');
  if (className) node.setAttribute('class', className);
  node.innerHTML = ICONS[name];
  return node;
}

let state = null, token = '', busy = false, exactPrompt = null;
let registry = { providers: [], selected: null, path: null };
let catalog = { groups: [] };
const ui = { language: 'ts', scene: 'normal', view: 'after', peeking: false, nudge: null, nudgeTab: 'color', semantic: false, feed: [], openMenu: null, live: null, reference: 'dark' };

const SNIPPETS = {
  ts: [
    [['syntax.comment', '// A deliberate change, not a replacement identity.']],
    [['syntax.keyword', 'const '], ['syntax.variable', 'reference'], ['text.primary', ' = '], ['syntax.string', '"something you love"'], ['text.primary', ';']],
    [],
    [['syntax.keyword', 'function '], ['syntax.function', 'refine'], ['text.primary', '('], ['syntax.variable', 'theme'], ['text.primary', ') {']],
    [['text.primary', '  '], ['syntax.keyword', 'const '], ['syntax.variable', 'candidate'], ['text.primary', ' = '], ['syntax.function', 'preserve'], ['text.primary', '('], ['syntax.variable', 'theme'], ['text.primary', ');']],
    [['text.primary', '  '], ['syntax.keyword', 'return '], ['syntax.function', 'validate'], ['text.primary', '('], ['syntax.variable', 'candidate'], ['text.primary', ');']],
    [['text.primary', '}']],
    [],
    [['syntax.comment', '// Selection, search and overlays need their own checks.']],
    [['syntax.type', 'Theme'], ['text.primary', ' · '], ['syntax.string', 'strings'], ['text.primary', ' · '], ['diagnostic.error.text', 'diagnostics']],
  ],
  rust: [
    [['syntax.comment', '// A deliberate change, not a replacement identity.']],
    [['syntax.keyword', 'let '], ['syntax.variable', 'reference'], ['text.primary', ' = '], ['syntax.function', 'load_theme'], ['text.primary', '('], ['syntax.string', '"something you love"'], ['text.primary', ');']],
    [],
    [['syntax.keyword', 'fn '], ['syntax.function', 'refine'], ['text.primary', '('], ['syntax.variable', 'theme'], ['text.primary', ': '], ['syntax.type', 'Theme'], ['text.primary', ') -> '], ['syntax.type', 'Theme'], ['text.primary', ' {']],
    [['text.primary', '    '], ['syntax.keyword', 'let '], ['syntax.variable', 'candidate'], ['text.primary', ' = '], ['syntax.function', 'preserve'], ['text.primary', '('], ['syntax.variable', 'theme'], ['text.primary', ');']],
    [['text.primary', '    '], ['syntax.function', 'validate'], ['text.primary', '('], ['syntax.variable', 'candidate'], ['text.primary', ')']],
    [['text.primary', '}']],
    [],
    [['syntax.comment', '// Selection, search and overlays need their own checks.']],
    [['syntax.keyword', 'let '], ['syntax.variable', 'report'], ['text.primary', ' = '], ['syntax.type', 'Result'], ['text.primary', ' · '], ['syntax.string', 'strings'], ['text.primary', ' · '], ['diagnostic.error.text', 'diagnostics'], ['text.primary', ';']],
  ],
  python: [
    [['syntax.comment', '# A deliberate change, not a replacement identity.']],
    [['syntax.variable', 'reference'], ['text.primary', ' = '], ['syntax.function', 'load'], ['text.primary', '('], ['syntax.string', '"something you love"'], ['text.primary', ')']],
    [],
    [['syntax.keyword', 'def '], ['syntax.function', 'refine'], ['text.primary', '('], ['syntax.variable', 'theme'], ['text.primary', '):']],
    [['text.primary', '    '], ['syntax.variable', 'candidate'], ['text.primary', ' = '], ['syntax.function', 'preserve'], ['text.primary', '('], ['syntax.variable', 'theme'], ['text.primary', ')']],
    [['text.primary', '    '], ['syntax.keyword', 'return '], ['syntax.function', 'validate'], ['text.primary', '('], ['syntax.variable', 'candidate'], ['text.primary', ')']],
    [],
    [],
    [['syntax.comment', '# Selection, search and overlays need their own checks.']],
    [['syntax.function', 'print'], ['text.primary', '('], ['syntax.type', 'Theme'], ['text.primary', ', '], ['syntax.string', '"strings"'], ['text.primary', ', '], ['diagnostic.error.text', 'diagnostics'], ['text.primary', ')']],
  ],
  go: [
    [['syntax.comment', '// A deliberate change, not a replacement identity.']],
    [['syntax.variable', 'reference'], ['text.primary', ' := '], ['syntax.function', 'load'], ['text.primary', '('], ['syntax.string', '"something you love"'], ['text.primary', ')']],
    [],
    [['syntax.keyword', 'func '], ['syntax.function', 'refine'], ['text.primary', '('], ['syntax.variable', 'theme'], ['text.primary', ' '], ['syntax.type', 'Theme'], ['text.primary', ') '], ['syntax.type', 'Theme'], ['text.primary', ' {']],
    [['text.primary', '\t'], ['syntax.variable', 'candidate'], ['text.primary', ' := '], ['syntax.function', 'preserve'], ['text.primary', '('], ['syntax.variable', 'theme'], ['text.primary', ')']],
    [['text.primary', '\t'], ['syntax.keyword', 'return '], ['syntax.function', 'validate'], ['text.primary', '('], ['syntax.variable', 'candidate'], ['text.primary', ')']],
    [['text.primary', '}']],
    [],
    [['syntax.comment', '// Selection, search and overlays need their own checks.']],
    [['syntax.type', 'fmt'], ['text.primary', '.'], ['syntax.function', 'Println'], ['text.primary', '('], ['syntax.string', '"Theme"'], ['text.primary', ', '], ['syntax.variable', 'strings'], ['text.primary', ', '], ['diagnostic.error.text', 'diagnostics'], ['text.primary', ')']],
  ],
};

const ROLE_NAMES = {
  'surface.base': 'Background', 'surface.panel': 'Panels', 'surface.float': 'Floating windows', 'surface.gutter': 'Gutter',
  'text.primary': 'Text', 'text.secondary': 'Secondary text',
  'syntax.comment': 'Comments', 'syntax.keyword': 'Keywords', 'syntax.string': 'Strings', 'syntax.function': 'Functions',
  'syntax.variable': 'Variables', 'syntax.type': 'Types',
  'interaction.selection': 'Selection', 'interaction.search': 'Search matches', 'interaction.search-current': 'Current match',
  'diagnostic.error.text': 'Errors', 'diff.added.line-bg': 'Added lines',
};
const PROPERTY_NAMES = {
  fg: 'Text color', bg: 'Background', special: 'Accent', fg_opacity: 'Text opacity', bg_opacity: 'Background opacity',
  special_opacity: 'Accent opacity', bold: 'Bold', italic: 'Italic', underline: 'Underline', strikethrough: 'Strikethrough', blend: 'Blend',
};
const COLOR_PROPERTIES = ['fg', 'bg', 'special'];
const PALETTE_ROLES = [
  ['surface.base', 'bg'], ['text.primary', 'fg'], ['syntax.comment', 'fg'], ['syntax.keyword', 'fg'], ['syntax.string', 'fg'],
  ['syntax.function', 'fg'], ['syntax.variable', 'fg'], ['syntax.type', 'fg'], ['diagnostic.error.text', 'fg'],
  ['surface.float', 'bg'], ['interaction.selection', 'bg'], ['interaction.search', 'bg'],
];
const SCENE_NAMES = { normal: 'Normal', selection: 'Selection', search: 'Search match', diff: 'Added line', float: 'Floating window' };

function titleCase(text) { return text.charAt(0).toUpperCase() + text.slice(1); }
function roleName(role) {
  if (role === '*') return 'Everything';
  if (ROLE_NAMES[role]) return ROLE_NAMES[role];
  return titleCase(role.split('.').slice(1).join(' ').replace(/[-_]/g, ' ') || role);
}
function shortModel(model) { return model.includes('/') ? model.slice(model.lastIndexOf('/') + 1) : model; }

function channel(value) { const v = value / 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }
function rgb(hex) {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex || '');
  if (!m) return null;
  const n = parseInt(m[1], 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}
function luminance(hex) { const c = rgb(hex); return c ? 0.2126 * channel(c[0]) + 0.7152 * channel(c[1]) + 0.0722 * channel(c[2]) : 0; }
function contrast(a, b) { const [x, y] = [luminance(a), luminance(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); }
function hue(hex) {
  const c = rgb(hex); if (!c) return 0;
  const [r, g, b] = c.map(channel);
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  const A = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
  const B = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
  return (Math.atan2(B, A) * 180 / Math.PI + 360) % 360;
}
function towards(hex, target, step) {
  const diff = ((target - hue(hex) + 540) % 360) - 180;
  const degrees = Math.sign(diff) * Math.min(step, Math.abs(diff));
  return Math.round(degrees * 10) / 10 || step;
}
function withAlpha(hex, alpha) { return hex + Math.round((alpha ?? 1) * 255).toString(16).padStart(2, '0'); }

function toast(message, good = false) {
  const node = $('toast');
  node.textContent = message;
  node.classList.toggle('good', good);
  node.hidden = false;
  clearTimeout(toast.timer);
  toast.timer = setTimeout(() => { node.hidden = true; }, good ? 3500 : 8000);
}

async function api(path, data = {}) {
  const response = await fetch('/api/' + path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'X-Hexwitch-Token': token },
    body: JSON.stringify({ revision: state?.revision, ...data }),
  });
  if (!response.ok) throw new Error('The engine rejected the request (' + response.status + '). Reload the page.');
  const body = await response.json();
  if (!body.ok) throw new Error(body.error);
  return body.result;
}
async function task(work) {
  if (busy) return;
  busy = true; render();
  try { await work(); } catch (error) { toast(error.message || String(error)); } finally { busy = false; render(); }
}
function update(next) { state = next; render(); }
function cursor() { return state ? state.history.findIndex(version => version.active) : 0; }
function addFeed(kind, text, meta) { ui.feed.push({ kind, text, meta, version: cursor() }); if (ui.feed.length > 200) ui.feed.shift(); }

function showingAfter() { return !!state?.candidate && ui.view === 'after' && !ui.peeking; }
function shownStyles() { return showingAfter() ? state.candidate_resolved : state.resolved; }
function styleOf(styles, role) { return styles?.[role] || {}; }

function render() {
  if (!state) return;
  renderTheme();
  renderPreview();
  renderPalette();
  renderTimeline();
  renderComposer();
  renderNudge();
  if (!$('model-menu').hidden) renderModelMenu();
  if ($('providers').open) renderProviders();
}

function chooseReference(id) {
  closeMenus(); closeNudge();
  task(async () => { update(await api('reference', { name: id })); ui.reference = id; ui.feed = []; render(); });
}
function renderThemes() {
  const query = $('theme-search').value.trim().toLowerCase();
  const nodes = [];
  for (const group of catalog.groups) {
    const themes = group.themes.filter(theme => !query || theme.name.toLowerCase().includes(query) || theme.mode.includes(query));
    if (!themes.length) continue;
    nodes.push(el('span', { class: 'menu-label', title: group.revision ? `${group.origin} @ ${group.revision.slice(0, 7)} · ${group.license}` : null, text: group.name }));
    for (const theme of themes) nodes.push(el('button', { type: 'button', role: 'menuitemradio', 'aria-checked': String(ui.reference === theme.id), 'data-reference': theme.id, onclick: () => chooseReference(theme.id) },
      el('span', { class: 'dots', 'aria-hidden': 'true' }, theme.dots.map(color => el('span', { style: `background:${color}` }))),
      el('span', { class: 'grow', text: theme.name }), el('span', { class: 'faint', text: theme.mode })));
  }
  if (!nodes.length) nodes.push(el('span', { class: 'list-note', text: 'No theme matches.' }));
  $('theme-list').replaceChildren(...nodes);
}

function renderTheme() {
  const styles = state.resolved;
  $('theme-name').textContent = state.current.meta.name;
  $('theme-dots').replaceChildren(...[
    styleOf(styles, 'surface.base').bg, styleOf(styles, 'syntax.keyword').fg, styleOf(styles, 'syntax.string').fg, styleOf(styles, 'syntax.type').fg,
  ].filter(Boolean).map(color => el('span', { style: `background:${color}` })));
}

function renderPreview() {
  const styles = shownStyles();
  const card = $('preview-card');
  const base = styleOf(styles, 'surface.base').bg || '#151c25';
  const primary = styleOf(styles, 'text.primary').fg || '#e5edf6';
  const scene = ui.scene;
  const background = scene === 'float' ? styleOf(styles, 'surface.float').bg || base : base;
  card.style.background = background;
  card.style.setProperty('--preview-fg', primary);
  card.style.setProperty('--preview-bg', background);
  card.classList.toggle('light-theme', luminance(background) > 0.4);
  const preview = $('preview');
  preview.style.color = primary;
  const changed = showingAfter() ? new Set(state.changes.map(change => change.role)) : new Set();
  const overlayRole = { selection: 'interaction.selection', search: 'interaction.search', diff: 'diff.added.line-bg' }[scene];
  const overlay = overlayRole ? styleOf(styles, overlayRole) : {};
  const lines = SNIPPETS[ui.language].map((tokens, index) => {
    const line = el('div', { class: 'code-line' }, el('span', { class: 'line-number', text: String(index + 1) }));
    const highlighted = index === 4 && overlay.bg;
    if (highlighted) line.style.background = withAlpha(overlay.bg, overlay.bg_opacity);
    for (const [role, text] of tokens) line.append(tokenNode(styles, role, text, primary, changed, index === 4 ? overlay : {}));
    return line;
  });
  preview.replaceChildren(...lines);
  $('compare').hidden = !state.candidate;
  $('space-hint').hidden = !state.candidate;
  for (const button of document.querySelectorAll('[data-view]')) button.setAttribute('aria-pressed', String((button.dataset.view === 'after') === showingAfter()));
}

function tokenNode(styles, role, text, primary, changed, overlay) {
  const style = styleOf(styles, role);
  const classes = ['token'];
  if (changed.has(role)) classes.push('changed');
  if (ui.nudge?.role === role) classes.push('focused');
  const span = el('span', { class: classes.join(' '), 'data-role': role, title: roleName(role), text });
  const foreground = overlay.fg || style.fg || primary;
  span.style.color = withAlpha(foreground, overlay.fg ? overlay.fg_opacity : style.fg_opacity);
  if (style.italic) span.style.fontStyle = 'italic';
  if (style.bold) span.style.fontWeight = '700';
  const decorations = [];
  if (style.underline) decorations.push('underline');
  if (style.strikethrough) decorations.push('line-through');
  if (decorations.length) span.style.textDecorationLine = decorations.join(' ');
  const decorationStyle = { curl: 'wavy', dot: 'dotted', dash: 'dashed', double: 'double' }[style.underline];
  if (decorationStyle) span.style.textDecorationStyle = decorationStyle;
  if (style.special) span.style.textDecorationColor = withAlpha(style.special, style.special_opacity);
  return span;
}

function renderPalette() {
  const styles = shownStyles();
  const changed = showingAfter() ? new Set(state.changes.map(change => change.role + ':' + change.property)) : new Set();
  const swatches = PALETTE_ROLES.filter(([role, property]) => styleOf(styles, role)[property]).map(([role, property]) => {
    const color = styleOf(styles, role)[property];
    return el('button', {
      type: 'button', class: 'swatch' + (changed.has(role + ':' + property) ? ' changed' : ''), style: `background:${color}`,
      title: `${roleName(role)} · ${color}`, 'aria-label': `${roleName(role)} ${PROPERTY_NAMES[property].toLowerCase()} ${color}`,
      onclick: event => openNudge(role, property, event.currentTarget),
    });
  });
  const more = el('button', { type: 'button', class: 'swatch more', title: 'All roles', 'aria-label': 'All roles', onclick: event => openRoles(event.currentTarget) }, icon('palette', 'fill'));
  $('palette').replaceChildren(...swatches, more);
}

function friendlyValue(value) {
  if (value == null) return '—';
  if (typeof value === 'boolean') return value ? 'on' : 'off';
  if (typeof value === 'number') return String(Math.round(value * 1000) / 1000);
  return String(value);
}
function valueNode(value, className) {
  const node = el('span', { class: 'value ' + className });
  if (typeof value === 'string' && rgb(value)) node.append(el('i', { style: `background:${value}` }));
  node.append(friendlyValue(value));
  return node;
}

function versionLabel(version, index) {
  if (index === 0) return ['Started from ', el('b', { text: version.label.includes(':') ? version.label.replace(/^Reference:\s*/, '') : state.current.meta.name })];
  return ['Kept ', el('b', { text: version.label })];
}
function versionRow(version, index, current, blocked) {
  const classes = ['version'];
  if (index === 0) classes.push('reference');
  if (index > current) classes.push('undone');
  const action = index === current
    ? el('span', { class: 'here', text: 'Now' })
    : blocked ? null : el('button', { type: 'button', class: 'text-button', text: index < current ? 'Go back' : 'Return', onclick: () => goTo(index) });
  return el('div', { class: classes.join(' ') }, el('span', { class: 'marker' }), el('span', { class: 'label' }, versionLabel(version, index)), action);
}
const FAILURES = {
  authentication: ['The provider rejected the key', 'Check the key in Providers & models.', true],
  rate_limited: ['The provider is rate limiting you', 'Wait a moment, then try again.'],
  timeout: ['No answer within 2 minutes', 'Try again, or pick a faster model.'],
  network: ["Couldn't reach the provider", 'Check the API root and your connection.', true],
  not_found: ['Model or endpoint not found', 'Check the model id, or the API root in Providers & models.', true],
  bad_request: ['The provider refused the request', 'Usually an unknown model id or an unsupported response format (edit the provider, then Advanced).', true],
  server_error: ['The provider had a server error', 'This is usually temporary. Try again.'],
  http: ['The provider returned an error', 'Try again, or pick another model.'],
  invalid_response: ["The model's answer wasn't usable", 'Try again, or pick a more capable model.'],
  refusal: ['The model declined', 'Rephrase the request and try again.'],
  truncated: ['The answer was cut off', 'Try again, or pick a model with a longer output limit.'],
  payload_too_large: ['The request or answer was too large', 'Try a shorter request.'],
  unavailable: ["The engine couldn't start curl", 'Install curl, then restart the engine.'],
};
function describeFailure(message) {
  const match = /^provider ([a-z_]+): (.*)$/.exec(message);
  if (match && FAILURES[match[1]]) {
    const [title, hint, providers] = FAILURES[match[1]];
    return { code: match[1], title, hint, detail: match[2], providers: !!providers };
  }
  if (/not set in the engine environment|pasted again/.test(message)) return { code: 'key', title: "The key isn't available", hint: message, detail: '', providers: true };
  return { code: 'other', title: 'The request failed', hint: '', detail: message, providers: false };
}
function failureNode(item) {
  const failure = item.failure;
  const latest = ui.feed[ui.feed.length - 1] === item && !state.candidate && !ui.live;
  return el('div', { class: 'card failure', role: 'alert' },
    el('div', { class: 'card-head' }, el('strong', { text: failure.title }), el('span', { class: 'faint', style: 'font-size:12px', text: item.meta || '' })),
    failure.hint ? el('p', { text: failure.hint }) : null,
    failure.detail ? el('code', { class: 'failure-detail', text: failure.detail }) : null,
    latest ? el('div', { class: 'failure-actions' },
      el('button', { type: 'button', class: 'ghost-button', disabled: busy, onclick: () => task(() => sendWords(item.retry.text, item.retry.answering, true)) }, icon('retry'), 'Try again'),
      el('button', { type: 'button', class: 'ghost-button', onclick: () => toggleMenu('model-button', 'model-menu', renderModelMenu) }, 'Change model'),
      failure.providers ? el('button', { type: 'button', class: 'ghost-button', onclick: () => openProviders(selection()?.provider.id) }, 'Providers') : null) : null);
}

function liveSeconds() {
  const live = ui.live;
  if (live.serverElapsed != null) return (live.serverElapsed + performance.now() - live.syncedAt) / 1000;
  return (performance.now() - live.started) / 1000;
}
function liveCard() {
  return el('div', { class: 'card thinking' },
    el('div', { class: 'card-head' },
      el('span', { class: 'thinking-title' }, el('span', { class: 'orbit', 'aria-hidden': 'true' }), el('strong', { text: `Asking ${shortModel(ui.live.model)}` })),
      el('span', { class: 'mono faint', id: 'live-elapsed', text: '0s' })),
    el('p', { id: 'live-phase', class: 'live-phase', role: 'status' }),
    el('div', { class: 'live-track', 'aria-hidden': 'true' }, el('span', { id: 'live-bar' })),
    el('div', { class: 'failure-actions' }, el('button', { type: 'button', class: 'ghost-button', id: 'live-cancel', onclick: cancelLive }, icon('stop'), 'Stop')));
}
function livePhase(seconds) {
  const live = ui.live;
  if (live.cancelling) return ['Stopping the request…', ''];
  if (live.lost) return ["Can't reach the engine. Is it still running in your terminal?", 'warn'];
  if (live.phase === 'checking') return ['Got an answer. Checking it against the theme…', ''];
  if (seconds >= 60) return [`Still waiting on ${live.host}. The engine gives up at 2 minutes.`, 'warn'];
  if (seconds >= 15) return [`Still waiting on ${live.host}. Larger models can take up to a minute.`, ''];
  return [`Sent to ${live.host}. Waiting for the model…`, ''];
}
function tickLive() {
  if (!ui.live || !$('live-elapsed')) return;
  const seconds = liveSeconds();
  $('live-elapsed').textContent = seconds < 60 ? `${Math.floor(seconds)}s` : `${Math.floor(seconds / 60)}m ${Math.floor(seconds % 60)}s`;
  const [text, tone] = livePhase(seconds);
  if ($('live-phase').textContent !== text) $('live-phase').textContent = text;
  $('live-phase').className = 'live-phase ' + tone;
  $('live-bar').style.width = Math.min(100, seconds / 120 * 100) + '%';
  $('live-cancel').disabled = !!ui.live.cancelling;
}
function startLive(chosen) {
  ui.live = { model: chosen.model, host: chosen.provider.host, started: performance.now(), phase: 'waiting', serverElapsed: null, syncedAt: 0, lost: false, cancelling: false, misses: 0 };
  ui.live.ticker = setInterval(tickLive, 250);
  ui.live.poller = setInterval(pollLive, 1000);
}
async function pollLive() {
  const live = ui.live;
  if (!live) return;
  try {
    const status = await (await fetch('/api/live')).json();
    if (ui.live !== live) return;
    live.lost = false; live.misses = 0;
    if (status.active) { live.phase = status.phase; live.serverElapsed = status.elapsed_ms; live.syncedAt = performance.now(); }
  } catch {
    if (ui.live !== live) return;
    live.misses += 1;
    live.lost = live.misses >= 2;
  }
  tickLive();
}
function stopLive() {
  if (!ui.live) return 0;
  const seconds = liveSeconds();
  clearInterval(ui.live.ticker);
  clearInterval(ui.live.poller);
  ui.live = null;
  return seconds;
}
async function cancelLive() {
  if (!ui.live || ui.live.cancelling) return;
  ui.live.cancelling = true;
  tickLive();
  renderComposer();
  for (let attempt = 0; attempt < 6 && ui.live; attempt += 1) {
    try {
      const response = await fetch('/api/live/cancel', { method: 'POST', headers: { 'Content-Type': 'application/json', 'X-Hexwitch-Token': token }, body: '{}' });
      if ((await response.json()).result?.cancelled) return;
    } catch {}
    await new Promise(resolve => setTimeout(resolve, 300));
  }
}

function feedNode(item) {
  if (item.kind === 'failure') return failureNode(item);
  if (item.kind === 'you') return el('div', { class: 'bubble', text: item.text });
  const meta = item.meta ? el('span', { class: 'meta' }, icon('hex'), item.meta) : null;
  return el('div', { class: 'reply ' + item.kind }, meta, el('p', { text: item.text }));
}

function welcomeCard() {
  return el('div', { class: 'card welcome' },
    el('h3', { text: 'Describe changes in plain words' }),
    el('p', { text: 'Connect a model and say what you want, like "make comments easier to read". Nudges and checks already work offline: click any token in the preview.' }),
    el('div', { class: 'card-actions' }, el('button', { type: 'button', class: 'primary', text: 'Connect a model', onclick: () => openWizard() })));
}

function questionCard() {
  const clarification = state.clarification;
  return el('div', { class: 'card question' },
    el('span', { class: 'question-kicker' }, icon('question'), 'One question before anything changes'),
    el('ol', { class: 'questions' }, clarification.questions.map(question => el('li', { text: question }))),
    el('div', {}, el('button', { type: 'button', class: 'text-button muted', text: 'Never mind', disabled: busy, onclick: () => task(discard) })));
}

function sceneLabel(scene) {
  const [kind, role] = scene.name.split('/');
  return [titleCase(kind || scene.name), role ? roleName(role) : ''].filter(Boolean).join(' · ');
}
function checkValue(scene) { return scene.contrast == null ? scene.status : scene.contrast.toFixed(2); }

const STATUS_RANK = { pass: 0, unverified: 1, fail: 2 };
function compareScenes(scenes) {
  const before = new Map((state.validation?.scenes || []).map(scene => [scene.name, scene]));
  const rank = scene => STATUS_RANK[scene?.status] ?? 0;
  const worse = scenes.filter(scene => rank(scene) > rank(before.get(scene.name)));
  const fixed = scenes.filter(scene => rank(scene) < rank(before.get(scene.name)));
  const failingBefore = scenes.filter(scene => scene.status === 'fail' && !worse.includes(scene));
  const unverified = scenes.filter(scene => scene.status === 'unverified' && !worse.includes(scene));
  return { before, worse, fixed, failingBefore, unverified };
}
function reviewSummary(comparison) {
  const parts = [];
  if (comparison.fixed.length) parts.push(`fixed ${comparison.fixed.length}`);
  if (comparison.failingBefore.length) parts.push(`${comparison.failingBefore.length} already failing`);
  if (comparison.unverified.length) parts.push(`${comparison.unverified.length} can't be verified`);
  return parts.length ? titleCase(parts.join(' · ')) : '';
}

function reviewCard() {
  const candidate = state.candidate;
  const validation = candidate.validation;
  const scenes = validation.scenes || [];
  const comparison = compareScenes(scenes);
  const risky = comparison.worse.length > 0;
  const status = !risky
    ? el('span', { class: 'status pass' }, icon('check'), validation.status === 'pass' ? `All ${scenes.length} checks pass` : 'No new problems')
    : el('span', { class: 'status ' + (comparison.worse.some(scene => scene.status === 'fail') ? 'fail' : 'warn') }, icon('alert'), `${comparison.worse.length} got worse`);
  const changedRoles = new Set(state.changes.map(change => change.role));
  const relevant = [...comparison.worse, ...comparison.fixed, ...scenes.filter(scene => scene.status === 'pass' && changedRoles.has(scene.name.split('/')[1]))];
  const shown = [...new Set(relevant)].slice(0, 4);
  const changes = state.changes.slice(0, 6).map(change => el('div', { class: 'change' },
    el('span', { class: 'change-label' }, roleName(change.role), change.property !== 'fg' ? el('span', { class: 'faint', text: '· ' + (PROPERTY_NAMES[change.property] || change.property) }) : null,
      change.semantic_rule != null ? el('span', { class: 'tag', text: 'semantic' }) : null,
      change.locked_after && !change.locked_before ? el('span', { class: 'tag', text: 'locked' }) : null),
    el('span', { class: 'change-values' }, valueNode(change.before?.[change.property], 'before'), icon('arrow'), valueNode(change.after?.[change.property], 'after'))));
  if (state.changes.length > 6) changes.push(el('span', { class: 'faint', text: `+ ${state.changes.length - 6} more` }));
  const summary = reviewSummary(comparison);
  const tile = scene => {
    const previous = comparison.before.get(scene.name);
    const value = previous?.contrast != null && scene.contrast != null && previous.contrast.toFixed(2) !== scene.contrast.toFixed(2)
      ? `${previous.contrast.toFixed(2)} → ${scene.contrast.toFixed(2)}` : checkValue(scene);
    return el('div', { class: 'check ' + scene.status, title: scene.reasons.join('\n') || scene.name }, el('span', { text: sceneLabel(scene) }), el('span', { text: value }));
  };
  return el('div', { class: 'card review' },
    el('div', { class: 'card-head' }, el('strong', { text: 'Ready to review' }), status),
    el('span', { class: 'faint', style: 'font-size:12.5px;margin-top:-8px', text: candidate.label }),
    el('div', { class: 'changes' }, changes.length ? changes : el('span', { class: 'faint', text: 'No visible property changes.' })),
    shown.length ? el('div', { class: 'checks' }, shown.map(tile)) : null,
    risky ? el('ul', { class: 'warning-list' }, comparison.worse.slice(0, 3).map(scene => el('li', { text: `${sceneLabel(scene)}: ${scene.reasons.join('; ') || scene.status}` }))) : null,
    summary || scenes.length ? el('details', { class: 'all-checks' }, el('summary', { text: summary ? `${summary} · all ${scenes.length} checks` : `All ${scenes.length} checks` }),
      el('ul', {}, scenes.map(scene => el('li', {}, el('span', { text: sceneLabel(scene) }), el('span', { class: 'mono', text: `${checkValue(scene)} · ${scene.status}` }))))) : null,
    el('div', { class: 'card-actions' },
      el('button', { type: 'button', class: 'ghost-button', id: 'reject', text: 'Discard', disabled: busy, onclick: () => task(discard) }),
      el('button', { type: 'button', class: 'primary' + (risky ? ' risky' : ''), id: 'accept', text: risky ? 'Keep anyway' : 'Keep change', disabled: busy, onclick: () => task(() => keep(validation.status !== 'pass')) })));
}

let lastTimelineSize = 0;
function renderTimeline() {
  const timeline = $('timeline');
  const current = cursor();
  const blocked = busy || !!state.candidate || !!state.clarification;
  ui.feed = ui.feed.filter(item => item.version < state.history.length);
  const nodes = [];
  state.history.forEach((version, index) => {
    nodes.push(versionRow(version, index, current, blocked));
    for (const item of ui.feed) if (item.version === index) nodes.push(feedNode(item));
  });
  if (!registry.providers.length && !state.candidate && !state.clarification && !ui.feed.some(item => item.kind === 'you')) nodes.push(welcomeCard());
  if (ui.live) nodes.push(liveCard());
  if (state.clarification && !ui.live) nodes.push(questionCard());
  if (state.candidate) nodes.push(reviewCard());
  const nearBottom = timeline.scrollHeight - timeline.scrollTop - timeline.clientHeight < 80;
  timeline.replaceChildren(...nodes);
  tickLive();
  const size = nodes.length;
  if (size > lastTimelineSize && lastTimelineSize) for (const node of nodes.slice(lastTimelineSize - size)) node.classList.add('enter');
  if (nearBottom || size !== lastTimelineSize) timeline.scrollTop = timeline.scrollHeight;
  lastTimelineSize = size;
  const kept = Math.max(0, current);
  $('session-meta').textContent = kept ? `${kept} kept · ⌘Z steps back` : '';
}

function selection() {
  const chosen = registry.selected;
  if (!chosen) return null;
  const provider = registry.providers.find(item => item.id === chosen.provider);
  return provider ? { provider, model: chosen.model } : null;
}
function keyProblem(provider) {
  if (provider.key_ready) return '';
  if (provider.key_source === 'env') return `${provider.key_env} isn't set in the engine's environment`;
  return 'Paste the key again; pasted keys are forgotten when the engine stops';
}

function renderComposer() {
  const chosen = selection();
  const dot = $('model-dot');
  dot.className = 'dot ' + (chosen ? (chosen.provider.key_ready ? 'ready' : 'missing') : 'offline');
  $('model-name').textContent = chosen ? shortModel(chosen.model) : 'Offline';
  $('model-button').title = chosen ? `${chosen.provider.name} · ${chosen.model}` : 'No model: shows the exact prompt';
  const request = $('request');
  request.placeholder = state.clarification ? 'Answer the question…' : state.candidate ? 'Describe something new (discards this change)…' : 'Describe a change…';
  const send = $('send');
  const live = !!ui.live;
  send.disabled = live ? !!ui.live.cancelling : busy || !request.value.trim();
  send.classList.toggle('busy', busy && !live);
  send.classList.toggle('stop', live);
  send.setAttribute('aria-label', live ? 'Stop the request' : 'Send');
  if (send.dataset.icon !== (live ? 'stop' : 'send')) { send.dataset.icon = live ? 'stop' : 'send'; send.replaceChildren(icon(live ? 'stop' : 'send')); }
  const note = $('composer-note');
  note.className = 'composer-note';
  if (!chosen) {
    note.replaceChildren(icon('offline'), 'Offline · shows the exact prompt instead of sending it',
      registry.providers.length ? null : el('button', { type: 'button', class: 'text-button', text: '· Connect a model', onclick: () => openWizard() }));
  } else if (!chosen.provider.key_ready) {
    note.classList.add('warn');
    note.replaceChildren(icon('alert'), keyProblem(chosen.provider), el('button', { type: 'button', class: 'text-button', text: '· Fix', onclick: () => openProviders(chosen.provider.id) }));
  } else {
    note.replaceChildren(icon('shield'), `Sends this theme and your words to ${chosen.provider.host}`);
  }
}

function renderModelMenu() {
  const menu = $('model-menu');
  const chosen = registry.selected;
  const groups = registry.providers.map(provider => el('div', { class: 'group', role: 'group', 'aria-label': provider.name },
    el('div', { class: 'group-head' }, el('span', { class: 'dot ' + (provider.key_ready ? 'ready' : 'missing') }), provider.name,
      provider.launch ? el('span', { class: 'faint', text: 'launch flags' }) : provider.kind === 'custom' ? el('span', { class: 'faint', text: 'custom' }) : null,
      provider.key_ready ? null : el('span', { class: 'faint', text: '· key missing' })),
    provider.models.length ? provider.models.map(model => {
      const active = chosen?.provider === provider.id && chosen?.model === model;
      return el('button', { type: 'button', class: 'menu-item', role: 'option', 'aria-selected': String(active), title: model, onclick: () => task(() => chooseModel(provider.id, model)) },
        el('span', { class: 'grow', style: 'overflow:hidden;text-overflow:ellipsis', text: model }),
        provider.default_model === model ? el('span', { class: 'default-tag', text: 'default' }) : null,
        active ? icon('check', 'tick') : null);
    }) : el('span', { class: 'list-note', text: 'No models yet' })));
  const offline = el('button', { type: 'button', class: 'menu-item offline-item', role: 'option', 'aria-selected': String(!chosen), onclick: () => task(() => chooseModel(null)) },
    icon('offline'), el('span', { class: 'grow' }, 'No model · offline', el('small', { text: 'Shows the exact prompt instead of sending it' })), !chosen ? icon('check', 'tick') : null);
  menu.replaceChildren(
    ...(groups.length ? groups : [el('div', { class: 'empty', text: 'No models connected yet.' })]),
    el('div', { class: 'group' }, el('div', { class: 'menu-rule' }), offline),
    el('div', { class: 'menu-foot' },
      el('button', { type: 'button', class: 'text-button', onclick: addModels }, editableProvider() ? 'Add models' : 'Connect a model'),
      el('button', { type: 'button', class: 'text-button muted', text: 'Manage providers', onclick: () => { closeMenus(); openProviders(); } })));
}
function editableProvider() {
  const current = selection()?.provider;
  return current && !current.launch ? current : registry.providers.find(provider => !provider.launch);
}
function addModels() {
  closeMenus();
  const provider = editableProvider();
  openWizard(provider ? { editId: provider.id } : {});
}
async function chooseModel(provider, model) {
  registry = await api('providers/select', provider ? { provider, model } : {});
  closeMenus();
}

function closeMenus() {
  for (const id of ['theme-menu', 'more-menu', 'scene-menu', 'model-menu', 'roles-menu']) $(id).hidden = true;
  for (const id of ['theme-button', 'more-button', 'scene-button', 'model-button']) $(id).setAttribute('aria-expanded', 'false');
  ui.openMenu = null;
}
function toggleMenu(buttonId, menuId, before) {
  const opening = $(menuId).hidden;
  closeMenus();
  if (!opening) return;
  if (before) before();
  $(menuId).hidden = false;
  $(buttonId).setAttribute('aria-expanded', 'true');
  ui.openMenu = menuId;
  $(menuId).querySelector('button, input')?.focus({ preventScroll: true });
}

function openRoles(anchor) {
  closeMenus(); closeNudge();
  const menu = $('roles-menu');
  menu.hidden = false;
  ui.openMenu = 'roles-menu';
  const rect = anchor.getBoundingClientRect();
  menu.style.left = Math.max(12, Math.min(rect.left, innerWidth - 292)) + 'px';
  menu.style.top = Math.max(12, rect.top - 370) + 'px';
  $('roles-search').value = '';
  renderRoles();
  $('roles-search').focus();
  openRoles.anchor = anchor;
}
function renderRoles() {
  const query = $('roles-search').value.trim().toLowerCase();
  const roles = Object.keys(state.current.styles).filter(role => !query || role.includes(query) || roleName(role).toLowerCase().includes(query));
  $('roles-list').replaceChildren(...roles.map(role => {
    const style = styleOf(shownStyles(), role);
    const property = style.fg ? 'fg' : style.bg ? 'bg' : 'fg';
    return el('button', { type: 'button', class: 'menu-item', onclick: () => { closeMenus(); openNudge(role, property, openRoles.anchor); } },
      el('span', { class: 'swatch', style: `width:16px;height:16px;background:${style[property] || 'transparent'}` }), roleName(role), el('span', { class: 'mono', text: role }));
  }));
}

function openNudge(role, property, anchor) {
  const rect = anchor instanceof Element ? anchor.getBoundingClientRect() : anchor;
  closeMenus();
  ui.nudge = { role, property };
  ui.nudgeTab = COLOR_PROPERTIES.includes(property) ? 'color' : 'style';
  const popover = $('nudge');
  popover.hidden = false;
  renderNudge();
  renderPreview();
  const width = popover.offsetWidth, height = popover.offsetHeight;
  let left = Math.min(Math.max(12, rect.left), innerWidth - width - 12);
  let top = rect.bottom + 10;
  if (top + height > innerHeight - 12) top = Math.max(12, rect.top - height - 10);
  popover.style.left = left + 'px';
  popover.style.top = top + 'px';
}
function closeNudge() {
  if (!ui.nudge) return;
  ui.nudge = null;
  $('nudge').hidden = true;
  if (state) renderPreview();
}

function renderNudge() {
  if (!ui.nudge) return;
  const { role, property } = ui.nudge;
  const styles = shownStyles();
  const style = styleOf(styles, role);
  $('nudge-title').textContent = roleName(role);
  $('nudge-role').textContent = role;
  for (const button of document.querySelectorAll('[data-nudge-tab]')) button.setAttribute('aria-selected', String(button.dataset.nudgeTab === ui.nudgeTab));
  for (const panel of document.querySelectorAll('[data-nudge-panel]')) panel.hidden = panel.dataset.nudgePanel !== ui.nudgeTab;
  const colorProperty = COLOR_PROPERTIES.includes(property) ? property : 'fg';
  const available = COLOR_PROPERTIES.filter(name => name === colorProperty || style[name] || name !== 'special');
  $('property-switch').replaceChildren(...available.map(name => el('button', {
    type: 'button', 'aria-pressed': String(name === colorProperty), text: PROPERTY_NAMES[name],
    onclick: () => { ui.nudge.property = name; renderNudge(); },
  })));
  const color = style[colorProperty];
  $('nudge-swatch').style.background = color || 'transparent';
  $('nudge-hex').textContent = color || 'inherited';
  const base = styleOf(styles, 'surface.base').bg;
  const contrastNode = $('nudge-contrast');
  contrastNode.className = 'faint';
  if (color && base && colorProperty === 'fg') {
    const ratio = contrast(color, base);
    contrastNode.textContent = `${ratio.toFixed(2)}:1 on background · ${ratio >= 4.5 ? 'readable' : 'hard to read'}`;
    contrastNode.className = ratio >= 4.5 ? 'pass' : 'warn';
  } else if (color && colorProperty === 'bg') {
    const text = styleOf(styles, 'text.primary').fg;
    contrastNode.textContent = text ? `Text on it: ${contrast(text, color).toFixed(2)}:1` : '';
  } else contrastNode.textContent = '';
  $('nudge-readable').hidden = colorProperty !== 'fg';
  if (document.activeElement !== $('exact-color')) $('exact-color').value = color || '';
  for (const button of document.querySelectorAll('[data-toggle]')) button.setAttribute('aria-pressed', String(!!style[button.dataset.toggle]));
  $('underline-style').value = typeof style.underline === 'string' ? style.underline : 'none';
  if (document.activeElement !== $('opacity-value')) $('opacity-value').value = style[colorProperty + '_opacity'] ?? 1;
  $('semantic-toggle').setAttribute('aria-pressed', String(ui.semantic));
  $('nudge-status').textContent = state.candidate ? `${state.changes.length} pending · keep or discard in Session` : 'Nudges stack until you keep them';
  for (const control of $('nudge').querySelectorAll('[data-nudge], [data-toggle], #exact-form button, #opacity-form button, #underline-style')) control.disabled = busy;
}

function target(role, property) { return { roles: [role], property, include_semantic: ui.semantic }; }
function colorProperty() { return COLOR_PROPERTIES.includes(ui.nudge.property) ? ui.nudge.property : 'fg'; }
function nudgeIntent(kind) {
  const { role } = ui.nudge;
  const property = colorProperty();
  const color = styleOf(shownStyles(), role)[property];
  const name = roleName(role);
  const suffix = property === 'fg' ? '' : ` ${PROPERTY_NAMES[property].toLowerCase()}`;
  const action = { target: target(role, property) };
  switch (kind) {
    case 'lighter': return [{ ...action, op: 'lightness', delta: 0.03 }, `${name}${suffix} lighter`];
    case 'darker': return [{ ...action, op: 'lightness', delta: -0.03 }, `${name}${suffix} darker`];
    case 'vivid': return [{ ...action, op: 'chroma', delta: 0.015 }, `${name}${suffix} more vivid`];
    case 'muted': return [{ ...action, op: 'chroma', delta: -0.015 }, `${name}${suffix} more muted`];
    case 'warmer': return [{ ...action, op: 'hue', degrees: towards(color, 60, 8) }, `${name}${suffix} warmer`];
    case 'cooler': return [{ ...action, op: 'hue', degrees: towards(color, 250, 8) }, `${name}${suffix} cooler`];
    case 'lock': return [{ ...action, op: 'lock', scope: 'persistent' }, `Lock ${name.toLowerCase()}${suffix}`];
    case 'unlock': return [{ ...action, op: 'unlock' }, `Unlock ${name.toLowerCase()}${suffix}`];
    case 'clear': return [{ ...action, op: 'state', state: 'clear' }, `Clear ${name.toLowerCase()}${suffix} override`];
    default: return null;
  }
}
function readableIntent() {
  const { role } = ui.nudge;
  const backgrounds = [{ surface: 'surface.base', overlays: [] }];
  return [{
    version: 1,
    actions: [{ op: 'contrast', target: target(role, 'fg'), min: 4.5, direction: 'both', backgrounds }],
    requirements: [{ role, property: 'fg', include_semantic: ui.semantic, min: 4.5, backgrounds, persistent: false }],
  }, `${roleName(role)} readable`];
}

function stackLabel(base, label) {
  const parts = base.split(' + ');
  const last = parts[parts.length - 1];
  const match = /^(.*) ×(\d+)$/.exec(last);
  const stem = match ? match[1] : last;
  if (stem === label) parts[parts.length - 1] = `${label} ×${(match ? Number(match[2]) : 1) + 1}`;
  else parts.push(label);
  return parts.join(' + ');
}
function mergeSpecs(base, extra) {
  const merged = { ...base, actions: [...(base.actions || []), ...(extra.actions || [])] };
  const requirements = [...(base.requirements || []), ...(extra.requirements || [])];
  if (requirements.length) merged.requirements = requirements;
  return merged;
}
function outcomeText(result) {
  const outcome = result?.outcome || 'unchanged';
  if (outcome === 'unchanged') return 'the theme already does that';
  const reason = result?.report?.refusals?.[0]?.reason || result?.report?.notes || '';
  return reason ? `${outcome}: ${typeof reason === 'string' ? reason : JSON.stringify(reason)}` : outcome;
}
async function propose(intent, label) {
  update(await api('revise', { spec: intent, label }));
  if (!state.candidate) addFeed('error', `Nothing changed: ${outcomeText(state.last_result)}. The engine inspector has details.`);
}
async function stack(intent, label) {
  const base = state.candidate;
  if (!base) return propose(intent, label);
  update(await api('reject'));
  try {
    await propose(mergeSpecs(base.spec, intent), stackLabel(base.label, label));
  } catch (error) {
    update(await api('revise', { spec: base.spec, label: base.label }));
    throw error;
  }
}
function runNudge(kind) {
  if (!ui.nudge) return;
  if (kind === 'readable') { const [intent, label] = readableIntent(); return task(() => stack(intent, label)); }
  const built = nudgeIntent(kind);
  if (built) task(() => stack({ version: 1, actions: [built[0]] }, built[1]));
}
function runToggle(name) {
  const { role } = ui.nudge;
  const current = !!styleOf(shownStyles(), role)[name];
  task(() => stack({ version: 1, actions: [{ op: 'set_bool', target: target(role, name), value: !current }] }, `${roleName(role)} ${current ? 'without' : 'with'} ${name}`));
}

async function keep(risky) {
  const label = state.candidate.label;
  update(await api('accept', { allow_unverified: risky }));
  toast(`Kept · ${label}`, true);
}
async function discard() { update(await api('reject')); }
async function goTo(index) {
  await task(async () => {
    let steps = index - cursor();
    while (steps !== 0) {
      update(await api(steps < 0 ? 'undo' : 'redo'));
      steps += steps < 0 ? 1 : -1;
    }
  });
}

async function sendRequest() {
  const text = $('request').value.trim();
  if (!text) return;
  $('request').value = '';
  autosize();
  await sendWords(text, !!state.clarification, false);
}
async function sendWords(text, answering, retrying) {
  const chosen = selection();
  if (state.candidate) { update(await api('reject')); addFeed('note', 'Discarded the pending change.'); }
  if (answering && !state.clarification) answering = false;
  if (!retrying) addFeed('you', text);
  const words = answering ? { answer: text } : { request: text };
  if (!chosen) {
    exactPrompt = await api('prompt', { mode: 'inspect', ...words });
    addFeed('note', 'Offline, so nothing was sent. The exact prompt a model would get is open in the engine inspector.');
    openInspector('prompt');
    return;
  }
  startLive(chosen);
  render();
  let result;
  try {
    result = await api('prompt', { mode: 'live', consent: true, provider: chosen.provider.id, model: chosen.model, ...words });
  } catch (error) {
    const cancelled = ui.live?.cancelling || /provider cancelled/.test(error.message);
    const seconds = stopLive().toFixed(1);
    try { update((await (await fetch('/api/bootstrap')).json()).state); } catch {}
    if (cancelled) addFeed('note', `Stopped after ${seconds}s. Nothing was changed.`);
    else ui.feed.push({ kind: 'failure', failure: describeFailure(error.message), meta: `${shortModel(chosen.model)} · ${seconds}s`, retry: { text, answering }, version: cursor() });
    return;
  }
  const seconds = stopLive().toFixed(1);
  update(result.state);
  const interpretation = result.interpretation;
  addFeed(interpretation.outcome === 'error' ? 'error' : 'engine', interpretation.notes || interpretation.outcome, `${shortModel(chosen.model)} · ${seconds}s`);
  if (interpretation.outcome !== 'error' && !state.candidate && !state.clarification) addFeed('note', `Nothing to change: ${outcomeText(state.last_result)}.`);
}
function autosize() {
  const area = $('request');
  area.style.height = 'auto';
  area.style.height = Math.min(area.scrollHeight, 180) + 'px';
  renderComposer();
}

const PRESETS = {
  openai: { name: 'OpenAI', base_url: 'https://api.openai.com/v1', key_env: 'OPENAI_API_KEY', mark: 'OA', blurb: 'api.openai.com' },
  openrouter: { name: 'OpenRouter', base_url: 'https://openrouter.ai/api/v1', key_env: 'OPENROUTER_API_KEY', mark: 'OR', blurb: 'Many vendors, one key' },
  local: { name: 'Local models', base_url: 'http://127.0.0.1:11434/v1', key_env: '', icon: 'monitor', title: 'On this machine', blurb: 'Ollama, LM Studio · no key' },
  custom: { name: '', base_url: '', key_env: '', icon: 'link', title: 'Any compatible API', blurb: 'Paste an API root, like a team gateway' },
};
let wiz = null;
function freshWizard(kind) {
  const preset = PRESETS[kind];
  return {
    step: 1, kind, name: preset.name, base_url: preset.base_url, key_env: preset.key_env, keyMode: kind === 'local' ? 'none' : 'env',
    key: '', response_format: 'json_schema', available: [], chosen: [], def: null, error: null, loading: false, query: '', envPresent: null, editId: null,
  };
}
function openWizard(options = {}) {
  closeMenus();
  closeNudge();
  if (options.editId) {
    const provider = registry.providers.find(item => item.id === options.editId);
    wiz = {
      ...freshWizard(provider.kind), step: 3, editId: provider.id, name: provider.name, base_url: provider.base_url, key_env: provider.key_env || '',
      keyMode: provider.key_source === 'env' ? 'env' : provider.key_source === 'none' ? 'none' : 'paste',
      response_format: provider.response_format, chosen: [...provider.models], def: provider.default_model, loading: true,
    };
    if ($('providers').open) $('providers').close();
    task(async () => {
      try { wiz.available = (await api('providers/test', { id: provider.id })).models; wiz.error = null; } catch (error) { wiz.error = error.message; }
      wiz.loading = false; renderWizard();
    });
  } else {
    if ($('providers').open) $('providers').close();
    wiz = freshWizard('openrouter');
  }
  renderWizard();
  if (!$('connect').open) $('connect').showModal();
}
function hostOf(url) { try { return new URL(url).host; } catch { return ''; } }
function wizardDraft() {
  return {
    id: wiz.editId || undefined, kind: wiz.kind, name: wiz.name.trim() || hostOf(wiz.base_url) || 'Provider', base_url: wiz.base_url.trim(),
    response_format: wiz.response_format, key_env: wiz.keyMode === 'env' ? wiz.key_env.trim() : undefined, key: wiz.keyMode === 'paste' && wiz.key ? wiz.key : undefined,
  };
}
function stepsNode() {
  const labels = [wiz.step > 1 ? wiz.name || 'Provider' : 'Provider', 'Key', 'Models'];
  const items = [];
  labels.forEach((label, index) => {
    const step = index + 1;
    const stateClass = step < wiz.step ? 'done' : step === wiz.step ? 'current' : '';
    if (index) items.push(el('li', { class: 'rule ' + (step <= wiz.step ? 'done' : ''), 'aria-hidden': 'true' }));
    items.push(el('li', { class: stateClass, 'aria-current': step === wiz.step ? 'step' : null }, el('span', { class: 'num' }, step < wiz.step ? icon('check') : String(step)), label));
  });
  return el('ol', { class: 'steps', 'aria-label': 'Steps' }, items);
}
function wizardShell(title, subtitle, body, foot) {
  return el('div', { class: 'wizard' },
    el('div', { class: 'wizard-head' }, stepsNode(), el('div', { class: 'wizard-title' }, el('h2', { id: 'connect-title', text: title }), subtitle ? el('p', {}, subtitle) : null)),
    el('div', { class: 'wizard-body' }, body),
    el('div', { class: 'wizard-foot' }, foot));
}
function renderWizard() {
  const dialog = $('connect');
  if (!wiz) return;
  const views = { 1: wizardProvider, 2: wizardKey, 3: wizardModels };
  const active = document.activeElement?.id;
  const position = document.activeElement?.selectionStart;
  dialog.replaceChildren(views[wiz.step]());
  if (active && $(active) && dialog.contains($(active))) {
    $(active).focus();
    if (position != null && $(active).setSelectionRange) try { $(active).setSelectionRange(position, position); } catch {}
  }
}
function wizardProvider() {
  const tiles = Object.entries(PRESETS).map(([kind, preset]) => el('button', {
    type: 'button', role: 'radio', class: 'tile' + (kind === 'custom' ? ' dashed' : ''), 'aria-checked': String(wiz.kind === kind),
    onclick: () => { wiz = freshWizard(kind); renderWizard(); }, ondblclick: () => { wiz = freshWizard(kind); wiz.step = 2; renderWizard(); checkEnv(); },
  }, el('span', { class: 'mark' }, preset.icon ? icon(preset.icon) : preset.mark), el('span', {}, el('strong', { text: preset.title || preset.name }), el('small', { text: preset.blurb })), el('span', { class: 'tick' }, icon('check'))));
  return wizardShell('Where should requests go?', 'Optional. Nudges and checks work offline. A model lets you describe changes in plain words.',
    el('div', { class: 'tiles', role: 'radiogroup', 'aria-label': 'Provider' }, tiles),
    [el('button', { type: 'button', class: 'text-button', text: registry.providers.length ? 'Cancel' : 'Stay offline for now', onclick: () => $('connect').close() }),
      el('button', { type: 'button', class: 'primary', id: 'wizard-next', onclick: () => { wiz.step = 2; renderWizard(); checkEnv(); } }, 'Continue', icon('arrow'))]);
}
let envTimer = 0;
function checkEnv() {
  clearTimeout(envTimer);
  if (wiz.keyMode !== 'env' || !/^[A-Za-z_][A-Za-z0-9_]*$/.test(wiz.key_env.trim())) { wiz.envPresent = null; return; }
  const name = wiz.key_env.trim();
  envTimer = setTimeout(async () => {
    try { const result = await api('providers/env', { name }); if (wiz && wiz.key_env.trim() === name) { wiz.envPresent = result.present; renderWizard(); } } catch {}
  }, 250);
}
function field(label, input) { return el('label', { class: 'field' }, el('span', { text: label }), input); }
function keyOption(mode, title, badge, body) {
  const active = wiz.keyMode === mode;
  return el('div', {
    class: 'option', role: 'radio', tabindex: active ? '0' : '-1', 'aria-checked': String(active),
    onclick: event => { if (wiz.keyMode !== mode && !event.target.closest('input')) { wiz.keyMode = mode; wiz.error = null; renderWizard(); checkEnv(); } },
    onkeydown: event => { if ((event.key === ' ' || event.key === 'Enter') && event.target === event.currentTarget) { event.preventDefault(); wiz.keyMode = mode; renderWizard(); checkEnv(); } },
  }, el('div', { class: 'option-head' }, el('span', { class: 'radio' }), el('strong', { text: title }), badge ? el('span', { class: 'badge', text: badge }) : null), body ? el('div', { class: 'option-body' }, body) : null);
}
function wizardKey() {
  const local = wiz.kind === 'local';
  const custom = wiz.kind === 'custom';
  const rootInput = el('input', { id: 'wizard-root', class: 'mono', value: wiz.base_url, spellcheck: 'false', placeholder: 'https://gateway.example.com/v1', oninput: event => { wiz.base_url = event.target.value; renderWizardFoot(); }, onchange: renderWizard });
  const nameInput = el('input', { id: 'wizard-name', value: wiz.name, placeholder: hostOf(wiz.base_url) || 'Team gateway', oninput: event => { wiz.name = event.target.value; } });
  const formatSelect = el('select', { id: 'wizard-format', onchange: event => { wiz.response_format = event.target.value; } },
    ['json_schema', 'json_object', 'none'].map(value => el('option', { value, text: value, selected: wiz.response_format === value })));
  const envBody = wiz.keyMode === 'env' ? [
    field('Environment variable', el('input', { id: 'wizard-env', class: 'mono', value: wiz.key_env, placeholder: 'MY_PROVIDER_API_KEY', spellcheck: 'false', autocomplete: 'off', oninput: event => { wiz.key_env = event.target.value; wiz.envPresent = null; checkEnv(); renderWizardFoot(); } })),
    wiz.envPresent === true ? el('span', { class: 'found yes' }, icon('check'), "Found in the engine's environment")
      : wiz.envPresent === false ? el('span', { class: 'found no' }, icon('alert'), `Not set. Run export ${wiz.key_env.trim()}=… and restart the engine, or paste the key instead.`) : null,
  ] : [el('small', { text: 'The engine reads it when a request goes out. Only the variable name is saved.' })];
  const pasteBody = wiz.keyMode === 'paste'
    ? [field('API key', el('input', { id: 'wizard-key', type: 'password', value: wiz.key, autocomplete: 'off', placeholder: wiz.editId ? 'Leave empty to keep the pasted key' : 'Paste your key', oninput: event => { wiz.key = event.target.value; renderWizardFoot(); } })),
      el('small', { text: 'Held in engine memory until you quit. Never written to disk or shown again.' })]
    : [el('small', { text: 'Held in engine memory until you quit. Never written to disk.' })];
  const options = [
    local ? keyOption('none', 'No key needed', 'Typical', [el('small', { text: 'Local servers like Ollama and LM Studio accept requests without a key.' })]) : null,
    keyOption('env', 'Read it from my shell', local ? null : 'Recommended', envBody),
    keyOption('paste', 'Paste it for this session', null, pasteBody),
  ];
  const advancedFields = el('div', { class: 'field-row' },
    custom || local ? null : field('API root', rootInput),
    custom ? null : field('Name', nameInput),
    field('Response format', formatSelect));
  const body = [
    custom || local ? el('div', { class: 'field-row' }, field('API root', rootInput), custom ? field('Name', nameInput) : null) : null,
    local ? el('small', { class: 'faint', text: 'Ollama listens on 127.0.0.1:11434/v1, LM Studio on 127.0.0.1:1234/v1.' }) : null,
    el('div', { role: 'radiogroup', 'aria-label': 'Key', style: 'display:flex;flex-direction:column;gap:10px' }, options),
    el('details', { class: 'advanced' }, el('summary', {}, icon('chevron'), el('span', { text: 'Advanced' }), el('span', { class: 'mono', text: `${hostOf(wiz.base_url) || 'no root yet'} · ${wiz.response_format}` })), advancedFields),
    el('div', { class: 'notice' }, icon('shield'), el('span', {}, 'When you press send, hexwitch sends the current theme and your words to ', el('b', { text: hostOf(wiz.base_url) || 'this provider' }), '. Loading the model list sends nothing else.')),
    wiz.error ? el('div', { class: 'notice error', role: 'alert' }, el('span', { text: wiz.error }), el('button', { type: 'button', class: 'text-button', text: 'Continue without the list, I\'ll type model ids', onclick: () => { wiz.error = null; wiz.step = 3; renderWizard(); } })) : null,
  ];
  const title = local ? 'Point to your local server' : custom ? 'Connect your API' : `Add your ${wiz.name} key`;
  return wizardShell(title, local ? 'Requests stay on this machine.' : 'The key stays with the local engine. This page never stores it.', body,
    [el('button', { type: 'button', class: 'text-button', text: 'Back', onclick: () => { wiz.step = 1; wiz.error = null; renderWizard(); } }),
      el('button', { type: 'button', class: 'primary', id: 'wizard-test', disabled: !keyStepReady() || wiz.loading, onclick: testConnection }, wiz.loading ? 'Testing…' : 'Test and load models', wiz.loading ? null : icon('arrow'))]);
}
function keyStepReady() {
  if (!wiz.base_url.trim()) return false;
  if (wiz.keyMode === 'env') return /^[A-Za-z_][A-Za-z0-9_]*$/.test(wiz.key_env.trim());
  if (wiz.keyMode === 'paste') return !!wiz.key || !!wiz.editId;
  return true;
}
function renderWizardFoot() {
  const button = $('wizard-test');
  if (button) button.disabled = !keyStepReady() || wiz.loading;
  const save = $('wizard-save');
  if (save) { save.disabled = !wiz.chosen.length || busy; save.textContent = wiz.chosen.length ? `Save ${wiz.chosen.length} model${wiz.chosen.length === 1 ? '' : 's'}` : 'Pick at least one model'; }
}
async function testConnection() {
  wiz.loading = true; wiz.error = null; renderWizard();
  try {
    wiz.available = (await api('providers/test', wizardDraft())).models;
    if (!wiz.chosen.length && wiz.available.length <= 3) wiz.chosen = [...wiz.available];
    wiz.step = 3;
  } catch (error) {
    wiz.error = error.message;
  }
  wiz.loading = false;
  renderWizard();
}
function toggleChosen(model) {
  wiz.chosen = wiz.chosen.includes(model) ? wiz.chosen.filter(item => item !== model) : [...wiz.chosen, model];
  if (!wiz.chosen.includes(wiz.def)) wiz.def = wiz.chosen[0] || null;
  renderWizard();
}
function wizardModels() {
  const query = wiz.query.trim().toLowerCase();
  const matches = wiz.available.filter(model => !query || model.toLowerCase().includes(query));
  const limit = 150;
  if (!wiz.def || !wiz.chosen.includes(wiz.def)) wiz.def = wiz.chosen[0] || null;
  const chips = wiz.chosen.map(model => el('span', { class: 'chosen-chip' + (model === wiz.def ? ' default' : '') },
    el('button', { type: 'button', class: 'star', 'aria-label': `Make ${model} the default`, title: 'Make default', onclick: () => { wiz.def = model; renderWizard(); } }, icon('star')),
    shortModel(model),
    el('button', { type: 'button', 'aria-label': `Remove ${model}`, onclick: () => toggleChosen(model) }, icon('close'))));
  const search = el('label', { class: 'search' }, icon('search'), el('input', {
    id: 'wizard-search', value: wiz.query, autocomplete: 'off', spellcheck: 'false', 'aria-label': 'Search models',
    placeholder: wiz.available.length ? `Search ${wiz.available.length} models, or type an exact id` : 'Type a model id and press Enter',
    oninput: event => { wiz.query = event.target.value; renderWizard(); },
    onkeydown: event => {
      if (event.key !== 'Enter') return;
      event.preventDefault();
      const typed = wiz.query.trim();
      if (!typed) return;
      const exact = wiz.available.find(model => model.toLowerCase() === typed.toLowerCase()) || (matches.length === 1 ? matches[0] : typed);
      if (!wiz.chosen.includes(exact)) wiz.chosen = [...wiz.chosen, exact];
      wiz.query = ''; renderWizard();
    },
  }));
  const rows = matches.slice(0, limit).map(model => el('button', { type: 'button', class: 'model-row', role: 'option', 'aria-selected': String(wiz.chosen.includes(model)), onclick: () => toggleChosen(model) },
    el('span', { class: 'box' }, icon('check')), el('span', { class: 'grow', text: model }), model === wiz.def ? el('span', { class: 'default-tag faint', text: 'Default' }) : null));
  const notes = [];
  if (wiz.loading) notes.push(el('div', { class: 'list-note', text: 'Loading models…' }));
  else if (wiz.error) notes.push(el('div', { class: 'notice error' }, el('span', { text: `Couldn't load the list: ${wiz.error}` })));
  if (matches.length > limit) notes.push(el('div', { class: 'list-note', text: `Showing ${limit} of ${matches.length}. Search to narrow.` }));
  if (query && !matches.length && !wiz.loading) notes.push(el('div', { class: 'list-note', text: `Press Enter to add "${wiz.query.trim()}".` }));
  else notes.push(el('div', { class: 'list-note', text: 'Not listed? Type its exact id and press Enter.' }));
  const connected = wiz.available.length ? el('span', { class: 'found yes' }, el('span', { class: 'dot ready' }), 'Connected') : null;
  return wizardShell('Build your model list', [connected, connected ? ' ' : null, "Pick the few you'll use. They show up under the model chip, and the starred one is the default."],
    [el('div', { class: 'chosen' }, chips.length ? chips : el('span', { class: 'faint', style: 'font-size:13px;align-self:center', text: 'Nothing picked yet.' })), search,
      el('div', { class: 'model-list', role: 'listbox', 'aria-multiselectable': 'true', 'aria-label': 'Available models' }, rows, notes)],
    [el('button', { type: 'button', class: 'text-button', text: wiz.editId ? 'Cancel' : 'Back', onclick: () => { if (wiz.editId) $('connect').close(); else { wiz.step = 2; renderWizard(); } } }),
      el('button', { type: 'button', class: 'primary', id: 'wizard-save', disabled: !wiz.chosen.length || busy, onclick: () => task(saveWizard) },
        wiz.chosen.length ? `Save ${wiz.chosen.length} model${wiz.chosen.length === 1 ? '' : 's'}` : 'Pick at least one model')]);
}
async function saveWizard() {
  const saved = await api('providers/save', { ...wizardDraft(), models: wiz.chosen, default_model: wiz.def || wiz.chosen[0] });
  registry = await api('providers/select', { provider: saved.id, model: wiz.def || wiz.chosen[0] });
  const provider = registry.providers.find(item => item.id === saved.id);
  $('connect').close();
  wiz = null;
  toast(`${provider.name} connected · ${shortModel(registry.selected.model)} is ready`, true);
}

let providersView = { current: null, confirm: null, paste: '' };
function openProviders(id) {
  closeMenus(); closeNudge();
  providersView = { current: id || registry.selected?.provider || registry.providers[0]?.id || null, confirm: null, paste: '' };
  renderProviders();
  if (!$('providers').open) $('providers').showModal();
}
function initials(name) { return name.split(/[\s.-]+/).filter(Boolean).slice(0, 2).map(part => part[0].toUpperCase()).join('') || '?'; }
function providerPayload(provider, changes = {}) {
  return {
    id: provider.id, kind: provider.kind, name: provider.name, base_url: provider.base_url, response_format: provider.response_format,
    key_env: provider.key_source === 'env' ? provider.key_env : undefined, models: provider.models, default_model: provider.default_model, ...changes,
  };
}
async function saveProvider(provider, changes) {
  const saved = await api('providers/save', providerPayload(provider, changes));
  registry = saved.providers;
}
function renderProviders() {
  const dialog = $('providers');
  const current = registry.providers.find(item => item.id === providersView.current) || registry.providers[0];
  const nav = el('nav', { class: 'provider-nav', 'aria-label': 'Providers' },
    registry.providers.map(provider => el('button', {
      type: 'button', class: 'provider-link', 'aria-current': String(provider === current), onclick: () => { providersView = { current: provider.id, confirm: null, paste: '' }; renderProviders(); },
    }, el('span', { class: 'mark' }, provider.kind === 'local' ? icon('monitor') : initials(provider.name)),
      el('span', { class: 'who' }, el('span', { text: provider.name }), provider.key_ready
        ? el('span', { text: `${provider.models.length} model${provider.models.length === 1 ? '' : 's'}${provider.launch ? ' · launch flags' : ''}` })
        : el('span', { class: 'warn', text: 'Key missing' })),
      el('span', { class: 'dot ' + (provider.key_ready ? 'ready' : 'missing') }))),
    el('button', { type: 'button', class: 'add-provider', onclick: () => openWizard() }, icon('plus'), 'Add provider'));
  const detail = current ? providerDetail(current) : el('section', { class: 'provider-detail' },
    el('div', { class: 'wizard-title' }, el('h2', { text: 'No providers yet', style: 'font-size:22px' }), el('p', { text: 'Connect one to describe changes in plain words. Keys never leave the local engine.' })),
    el('div', {}, el('button', { type: 'button', class: 'primary', text: 'Connect a model', onclick: () => openWizard() })));
  dialog.replaceChildren(
    el('div', { class: 'providers-head' }, el('h2', { id: 'providers-title', text: 'Providers & models' }),
      el('button', { type: 'button', class: 'icon-button', 'aria-label': 'Close', onclick: () => dialog.close() }, icon('close'))),
    el('div', { class: 'providers-layout' }, nav, detail));
}
function providerDetail(provider) {
  const readOnly = provider.launch;
  const keyField = provider.key_source === 'env'
    ? el('div', { class: 'readonly' }, el('span', { text: provider.key_env }), provider.key_ready ? icon('check') : icon('alert'))
    : provider.key_source === 'none' ? el('div', { class: 'readonly' }, el('span', { text: 'No key' }))
      : el('div', { class: 'readonly' }, el('span', { text: provider.key_ready ? 'Pasted for this session' : 'Needs pasting again' }), provider.key_ready ? icon('check') : icon('alert'));
  const paste = provider.key_source === 'session' && !provider.key_ready ? el('form', { class: 'paste-row', onsubmit: event => { event.preventDefault(); task(async () => { await saveProvider(provider, { key: providersView.paste }); providersView.paste = ''; toast('Key held for this session', true); }); } },
    el('label', { class: 'sr-only', for: 'paste-key', text: 'API key' }),
    el('input', { id: 'paste-key', type: 'password', placeholder: 'Paste the key again', autocomplete: 'off', value: providersView.paste, oninput: event => { providersView.paste = event.target.value; } }),
    el('button', { type: 'submit', class: 'primary', text: 'Save' })) : null;
  const models = provider.models.length ? provider.models.map(model => el('div', { class: 'model-line' },
    readOnly ? el('span', { style: 'width:32px' }) : el('button', {
      type: 'button', class: 'star' + (model === provider.default_model ? ' on' : ''), 'aria-label': `Make ${model} the default`, title: 'Make default',
      onclick: () => task(() => saveProvider(provider, { default_model: model })),
    }, icon('star')),
    el('span', { class: 'grow', text: model }), model === provider.default_model ? el('span', { class: 'default-tag', text: 'Default' }) : null))
    : [el('div', { class: 'list-note', text: 'No models yet. Use Edit list to add some.' })];
  const confirming = providersView.confirm === provider.id;
  return el('section', { class: 'provider-detail' },
    el('div', { class: 'detail-title' },
      el('div', {}, el('strong', { text: provider.name }), provider.key_ready
        ? el('span', { class: 'ready-line yes' }, el('span', { class: 'dot ready' }), 'Ready')
        : el('span', { class: 'ready-line no' }, icon('alert'), keyProblem(provider))),
      el('button', { type: 'button', class: 'ghost-button', text: 'Test connection', disabled: busy, onclick: () => task(async () => { const result = await api('providers/test', { id: provider.id }); toast(`${provider.name} answered · ${result.models.length} models available`, true); }) })),
    el('div', { class: 'field-row' }, el('div', { class: 'field' }, el('span', { text: 'API root' }), el('div', { class: 'readonly' }, el('span', { text: provider.base_url }))),
      el('div', { class: 'field' }, el('span', { text: provider.key_source === 'env' ? 'Key · from environment' : 'Key' }), keyField)),
    paste,
    el('div', { class: 'section-head' }, el('strong', { text: 'Your models' }), readOnly ? null : el('button', { type: 'button', class: 'text-button', text: 'Edit list', onclick: () => openWizard({ editId: provider.id }) })),
    el('div', { class: 'models-box' }, models),
    el('div', { class: 'detail-foot' },
      el('span', { text: readOnly ? 'From the engine\'s launch flags · read-only' : registry.path ? `Saved in ${registry.path} · keys are never saved` : 'Kept in memory for this run · keys are never saved' }),
      readOnly ? null : el('button', {
        type: 'button', class: 'text-button danger', text: confirming ? 'Click again to remove' : 'Remove provider', disabled: busy,
        onclick: () => {
          if (!confirming) { providersView.confirm = provider.id; renderProviders(); return; }
          task(async () => { registry = await api('providers/remove', { id: provider.id }); providersView = { current: registry.providers[0]?.id || null, confirm: null, paste: '' }; toast(`${provider.name} removed`, true); });
        },
      })));
}

function openInspector(kind) {
  if (kind) $('inspect-kind').value = kind;
  inspector();
  if (!$('inspector-dialog').open) $('inspector-dialog').showModal();
}
function inspector() {
  const kind = $('inspect-kind').value;
  const value = kind === 'prompt' ? exactPrompt : state?.[kind];
  $('inspect-output').textContent = value == null ? (kind === 'prompt' ? 'Send a request while offline to capture the exact prompt.' : 'Nothing yet.') : JSON.stringify(value, null, 2);
}

function download(name, data) {
  const url = URL.createObjectURL(new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' }));
  const link = el('a', { href: url, download: name });
  document.body.append(link); link.click(); link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
function exportSession() {
  task(async () => {
    const bundle = await api('export');
    download(`hexwitch-session-${new Date().toISOString().slice(0, 19).replace(/[:T]/g, '-')}.json`, bundle);
    toast('Session saved to your downloads', true);
  });
}
function readFile(input, limit, work) {
  const file = input.files[0];
  input.value = '';
  if (!file) return;
  task(async () => {
    if (file.size > limit) throw new Error('That file is larger than 8 MiB.');
    await work(await file.text());
  });
}

function bind() {
  $('theme-button').addEventListener('click', () => toggleMenu('theme-button', 'theme-menu', () => { $('theme-search').value = ''; renderThemes(); }));
  $('more-button').addEventListener('click', () => toggleMenu('more-button', 'more-menu'));
  $('scene-button').addEventListener('click', () => toggleMenu('scene-button', 'scene-menu', () => {
    for (const button of document.querySelectorAll('[data-scene]')) button.setAttribute('aria-checked', String(button.dataset.scene === ui.scene));
  }));
  $('model-button').addEventListener('click', () => toggleMenu('model-button', 'model-menu', renderModelMenu));
  $('theme-search').addEventListener('input', renderThemes);
  $('theme-search').addEventListener('keydown', event => {
    if (event.key === 'Enter') { event.preventDefault(); $('theme-list').querySelector('button')?.click(); }
  });
  for (const button of document.querySelectorAll('[data-scene]')) button.addEventListener('click', () => {
    ui.scene = button.dataset.scene; $('scene-name').textContent = SCENE_NAMES[ui.scene]; closeMenus(); renderPreview();
  });
  for (const button of document.querySelectorAll('[data-language]')) button.addEventListener('click', () => {
    ui.language = button.dataset.language;
    for (const other of document.querySelectorAll('[data-language]')) other.setAttribute('aria-selected', String(other === button));
    renderPreview();
  });
  for (const button of document.querySelectorAll('[data-view]')) button.addEventListener('click', () => { ui.view = button.dataset.view; render(); });
  const actions = {
    export: exportSession,
    restore: () => $('restore').click(),
    import: () => $('import').click(),
    contrast: () => $('contrast-dialog').showModal(),
    providers: () => openProviders(),
    inspector: () => openInspector(),
    replay: () => { $('replay-request').value = $('request').value; $('replay-dialog').showModal(); },
    matrix: () => $('matrix-dialog').showModal(),
  };
  for (const button of document.querySelectorAll('[data-action]')) button.addEventListener('click', () => { closeMenus(); actions[button.dataset.action](); });

  $('preview').addEventListener('click', event => {
    const token = event.target.closest('.token');
    if (token) return openNudge(token.dataset.role, 'fg', token);
    const role = ui.scene === 'float' ? 'surface.float' : 'surface.base';
    openNudge(role, 'bg', { left: event.clientX, right: event.clientX, top: event.clientY, bottom: event.clientY });
  });
  $('nudge-close').addEventListener('click', closeNudge);
  for (const button of document.querySelectorAll('[data-nudge-tab]')) button.addEventListener('click', () => { ui.nudgeTab = button.dataset.nudgeTab; renderNudge(); });
  for (const button of document.querySelectorAll('[data-nudge]')) button.addEventListener('click', () => runNudge(button.dataset.nudge));
  for (const button of document.querySelectorAll('[data-toggle]')) button.addEventListener('click', () => runToggle(button.dataset.toggle));
  $('semantic-toggle').addEventListener('click', () => { ui.semantic = !ui.semantic; renderNudge(); });
  $('exact-form').addEventListener('submit', event => {
    event.preventDefault();
    const value = $('exact-color').value.trim();
    if (!/^#[0-9a-f]{6}$/i.test(value)) return toast('Use a six-digit hex color like #92a2b7.');
    const { role } = ui.nudge;
    const property = colorProperty();
    task(() => stack({ version: 1, actions: [{ op: 'set_color', target: target(role, property), color: value.toLowerCase() }] }, `${roleName(role)} ${value.toLowerCase()}`));
  });
  $('underline-style').addEventListener('change', event => {
    const { role } = ui.nudge;
    const value = event.target.value;
    const action = value === 'none' ? { op: 'set_bool', target: target(role, 'underline'), value: false } : { op: 'set_underline', target: target(role, 'underline'), value };
    task(() => stack({ version: 1, actions: [action] }, `${roleName(role)} underline ${value}`));
  });
  $('opacity-form').addEventListener('submit', event => {
    event.preventDefault();
    const value = Number($('opacity-value').value);
    if (!Number.isFinite(value) || value < 0 || value > 1) return toast('Opacity is a number from 0 to 1.');
    const { role } = ui.nudge;
    const property = colorProperty() + '_opacity';
    task(() => stack({ version: 1, actions: [{ op: 'set_number', target: target(role, property), value }] }, `${roleName(role)} ${PROPERTY_NAMES[property].toLowerCase()} ${value}`));
  });
  $('roles-search').addEventListener('input', renderRoles);

  $('composer').addEventListener('submit', event => { event.preventDefault(); closeMenus(); if (ui.live) cancelLive(); else task(sendRequest); });
  $('request').addEventListener('input', autosize);
  $('request').addEventListener('keydown', event => {
    if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) { event.preventDefault(); $('composer').requestSubmit(); }
  });

  $('import').addEventListener('change', event => readFile(event.target, 8 * 1024 * 1024, async source => {
    update(await api('import', { source })); ui.reference = null; ui.feed = []; render(); toast(`Imported ${state.current.meta.name}`, true);
  }));
  $('restore').addEventListener('change', event => readFile(event.target, 8 * 1024 * 1024, async source => {
    update(await api('restore', { bundle: JSON.parse(source) })); ui.reference = null; ui.feed = []; render(); toast('Session restored', true);
  }));
  $('measure').addEventListener('click', () => task(async () => {
    const result = await api('color', { foreground: $('color-fg').value, background: $('color-bg').value });
    const lch = result.oklch;
    $('color-result').textContent = `${result.ratio.toFixed(2)}:1  ${result.ratio >= 7 ? 'AAA' : result.ratio >= 4.5 ? 'AA' : result.ratio >= 3 ? 'large text only' : 'too low'}\nOKLCH ${[lch.l, lch.c, lch.h].map(n => Number(n).toFixed(3)).join(' ')}`;
  }));
  $('inspect-kind').addEventListener('change', inspector);
  $('submit-spec').addEventListener('click', () => task(async () => {
    const spec = JSON.parse($('spec').value);
    if (state.candidate) update(await api('reject'));
    await propose(spec, 'Explicit IntentSpec');
    inspector();
  }));
  $('replay-run').addEventListener('click', () => task(async () => {
    const text = $('replay-request').value.trim();
    const answering = !!state.clarification;
    if (state.candidate) update(await api('reject'));
    const result = await api('prompt', { mode: 'replay', response: $('replay-response').value, ...(answering ? { answer: text } : { request: text }) });
    addFeed('you', text || '(empty request)');
    update(result.state);
    addFeed(result.interpretation.outcome === 'error' ? 'error' : 'engine', result.interpretation.notes || result.interpretation.outcome, 'replay');
    $('replay-dialog').close();
  }));
  $('evaluate').addEventListener('click', () => task(async () => {
    const result = await api('evaluate');
    $('eval-output').textContent = JSON.stringify({ kind: result.kind, passed: result.passed, failed: result.failed, pairs: result.pairs, pair_failures: result.pair_failures, failures: result.cases?.filter(c => !c.passed) }, null, 2);
  }));
  $('toast').addEventListener('click', () => { $('toast').hidden = true; });
  $('connect').addEventListener('close', () => { wiz = null; render(); });

  document.addEventListener('pointerdown', event => {
    if (ui.openMenu && !event.target.closest('.menu') && !event.target.closest('[aria-haspopup]') && !event.target.closest('.swatch.more')) closeMenus();
    if (ui.nudge && !event.target.closest('#nudge') && !event.target.closest('.token') && !event.target.closest('.swatch') && !event.target.closest('#preview') && !event.target.closest('#roles-menu') && !event.target.closest('#toast')) closeNudge();
  });
  document.addEventListener('keydown', event => {
    const origin = event.target instanceof Element ? event.target : document.body;
    const typing = origin.closest('textarea, input, select, [contenteditable]');
    if (event.key === 'Escape' && !document.querySelector('dialog[open]')) { closeMenus(); closeNudge(); return; }
    if (event.code === 'Space' && !typing && !origin.closest('button, a, [role="radio"]') && state?.candidate) {
      event.preventDefault();
      if (!ui.peeking) { ui.peeking = true; renderPreview(); renderPalette(); }
      return;
    }
    if (!(event.metaKey || event.ctrlKey) || event.altKey || document.querySelector('dialog[open]')) return;
    const key = event.key.toLowerCase();
    if (key === 's') { event.preventDefault(); exportSession(); return; }
    if (typing || !state || state.candidate || state.clarification) return;
    if (key === 'z' && !event.shiftKey && state.can_undo) { event.preventDefault(); task(async () => update(await api('undo'))); }
    else if (((key === 'z' && event.shiftKey) || key === 'y') && state.can_redo) { event.preventDefault(); task(async () => update(await api('redo'))); }
  });
  document.addEventListener('keyup', event => {
    if (event.code === 'Space' && ui.peeking) { ui.peeking = false; renderPreview(); renderPalette(); }
  });
  window.addEventListener('blur', () => { if (ui.peeking) { ui.peeking = false; render(); } });
}

(async () => {
  bind();
  try {
    const boot = await (await fetch('/api/bootstrap')).json();
    token = boot.token;
    registry = boot.providers;
    catalog = boot.themes || catalog;
    update(boot.state);
  } catch (error) {
    toast(error.message || String(error));
  }
})();
