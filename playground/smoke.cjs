#!/usr/bin/env node
const assert = require('node:assert/strict');
const { spawn } = require('node:child_process');
const { once } = require('node:events');
const http = require('node:http');
const os = require('node:os');
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require('playwright');

const live = process.argv.includes('--live');
const binary = process.env.HEXWITCH_BIN || path.resolve(__dirname, '../core/target/debug/hexwitch');
const providersFile = path.join(os.tmpdir(), `hexwitch-smoke-providers-${process.pid}.json`);
const args = ['playground', '--port', '0', '--providers-file', providersFile];
if (live) {
  for (const [env, flag] of [['OPENAI_BASE_URL', '--base-url'], ['HEXWITCH_API_KEY_ENV', '--api-key-env'], ['HEXWITCH_MODEL', '--model']]) {
    assert.ok(process.env[env], `${env} required for --live`); args.push(flag, process.env[env]);
  }
}

const READY = { version: 1, decision: 'ready', notes: 'Darken comments slightly.', questions: [], actions: [{ op: 'lightness', roles: ['syntax.comment'], property: 'fg', include_semantic: false, scope: null, color: null, number: -0.02, boolean: null, underline: null, state: null, backgrounds: null, min: null, direction: null, persistent: null }] };
const mock = http.createServer((request, response) => {
  if (request.headers.authorization !== 'Bearer smoke-key') { response.writeHead(401); response.end('{}'); return; }
  let body = '';
  request.on('data', chunk => body += chunk);
  request.on('end', () => {
    if (body.includes('HANG')) return;
    if (body.includes('BADMODEL')) { response.writeHead(400, { 'Content-Type': 'application/json' }); response.end('{}'); return; }
    response.writeHead(200, { 'Content-Type': 'application/json' });
    if (request.method === 'GET') response.end(JSON.stringify({ data: [{ id: 'smoke/fast' }, { id: 'smoke/slow' }] }));
    else response.end(JSON.stringify({ choices: [{ finish_reason: 'stop', message: { content: JSON.stringify(READY) } }] }));
  });
});

const host = spawn(binary, args, { stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env, SMOKE_PROVIDER_KEY: 'smoke-key' } });
let stderr = ''; host.stderr.on('data', chunk => stderr += chunk.toString());
let activePage;
async function exportFailure() {
  if (!activePage || !process.env.HEXWITCH_FAILURE_BUNDLE) return;
  try {
    const bundle = await activePage.evaluate(async () => {
      const boot = await (await fetch('/api/bootstrap')).json();
      const result = await (await fetch('/api/export', { method: 'POST', headers: { 'Content-Type': 'application/json', 'X-Hexwitch-Token': boot.token }, body: '{}' })).json();
      return result.result;
    });
    fs.mkdirSync(path.dirname(process.env.HEXWITCH_FAILURE_BUNDLE), { recursive: true });
    fs.writeFileSync(process.env.HEXWITCH_FAILURE_BUNDLE, JSON.stringify(bundle, null, 2));
    console.log('Failure experiment exported to ' + process.env.HEXWITCH_FAILURE_BUNDLE);
  } catch {}
}

(async () => {
  let browser;
  try {
    mock.listen(0, '127.0.0.1'); await once(mock, 'listening');
    const mockRoot = `http://127.0.0.1:${mock.address().port}/v1`;
    const address = await new Promise((resolve, reject) => {
      let output = ''; host.stdout.on('data', chunk => { output += chunk; const match = output.match(/http:\/\/127\.0\.0\.1:\d+/); if (match) resolve(match[0]); });
      host.on('exit', code => reject(new Error(`Host exited ${code}: ${stderr}`)));
    });
    browser = await chromium.launch({ headless: true, ...(process.env.HEXWITCH_CHROME ? { executablePath: process.env.HEXWITCH_CHROME } : {}) });
    const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
    const page = await context.newPage(); activePage = page; const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.goto(address); await page.waitForSelector('#preview .token');

    const snapshot = () => page.evaluate(async () => (await (await fetch('/api/bootstrap')).json()));
    const state = async () => (await snapshot()).state;
    const idle = () => page.waitForFunction(() => !document.querySelector('#send').classList.contains('busy'));
    async function click(selector) { await page.locator(selector).first().click(); await idle(); }
    async function nudgeToken(role) { await page.locator(`#preview .token[data-role="${role}"]`).first().click(); await page.waitForSelector('#nudge:not([hidden])'); }
    async function closeNudge() { await page.keyboard.press('Escape'); await page.waitForSelector('#nudge', { state: 'hidden' }); }
    async function blur() { await page.evaluate(() => document.activeElement?.blur()); }
    async function more(action) { await page.locator('#more-button').click(); await page.locator(`#more-menu [data-action="${action}"]`).click(); }

    await page.locator('#preview').click({ position: { x: 600, y: 360 } });
    await page.waitForSelector('#nudge:not([hidden])');
    assert.equal(await page.locator('#nudge-role').textContent(), 'surface.base');
    await click('[data-nudge="darker"]');
    let current = await state();
    assert.ok(current.candidate); assert.equal(current.changes.length, 1);
    assert.equal(current.changes[0].role, 'surface.base'); assert.equal(current.current.styles['surface.base'].bg, '#151c25');
    await closeNudge();
    const after = await page.locator('#preview-card').evaluate(el => el.style.background);
    await page.locator('[data-view="before"]').click();
    assert.notEqual(await page.locator('#preview-card').evaluate(el => el.style.background), after);
    await page.locator('[data-view="after"]').click();
    await click('#accept');
    await blur(); await page.keyboard.press('ControlOrMeta+z'); await idle();
    assert.equal((await state()).current.styles['surface.base'].bg, '#151c25');
    await blur(); await page.keyboard.press('ControlOrMeta+Shift+z'); await idle();
    assert.notEqual((await state()).current.styles['surface.base'].bg, '#151c25');

    await nudgeToken('syntax.keyword');
    await page.locator('#semantic-toggle').click();
    await page.locator('[data-nudge-tab="style"]').click();
    await click('[data-toggle="italic"]');
    current = await state();
    assert.equal(current.candidate.theme.styles['syntax.keyword'].italic, false);
    assert.ok(current.changes.some(change => change.semantic_rule != null));
    await page.locator('#semantic-toggle').click();
    await closeNudge(); await click('#reject');

    await nudgeToken('syntax.comment');
    await click('#nudge-readable');
    current = await state();
    assert.ok(current.candidate.report.requirements.length > 0);
    assert.ok(current.candidate.validation.scenes.find(scene => scene.name === 'normal/syntax.comment').contrast >= 4.5);
    assert.equal(await page.locator('#accept').textContent(), 'Keep change');
    await click('[data-nudge="lighter"]'); await click('[data-nudge="lighter"]');
    assert.match((await state()).candidate.label, /lighter ×2/);
    await closeNudge(); await click('#reject');

    await more('matrix'); await click('#evaluate');
    assert.match(await page.locator('#eval-output').textContent(), /"passed": 74/);
    await page.locator('#matrix-dialog [value="close"]').click();

    await page.locator('#theme-button').click(); await page.locator('[data-reference="light"]').click();
    await page.waitForFunction(() => document.querySelector('#theme-name').textContent === 'Eval paper');
    await page.locator('#theme-button').click(); await page.fill('#theme-search', 'tokyo'); await page.keyboard.press('Enter');
    await page.waitForFunction(() => document.querySelector('#theme-name').textContent === 'Tokyo Night (Omarchy)');
    assert.equal((await state()).current.styles['syntax.keyword'].fg, '#bb9af7');
    await page.locator('#theme-button').click(); await page.locator('[data-reference="dark"]').click();
    await page.waitForFunction(() => document.querySelector('#theme-name').textContent === 'Eval ink');

    await more('replay');
    await page.fill('#replay-request', 'Make that blue quieter');
    await page.fill('#replay-response', JSON.stringify({ version: 1, decision: 'needs_clarification', notes: 'Choose a target.', questions: ['Comments or keywords?'], actions: [] }));
    await click('#replay-run');
    assert.ok((await state()).clarification);
    await page.waitForSelector('.card.question');
    await more('replay');
    await page.fill('#replay-request', 'Comments');
    await page.fill('#replay-response', JSON.stringify({ ...READY, actions: [{ ...READY.actions[0], op: 'chroma' }] }));
    await click('#replay-run');
    assert.ok((await state()).candidate); await click('#reject');

    if (!live) {
      await page.fill('#request', 'Make the background darker'); await page.keyboard.press('Enter'); await idle();
      await page.waitForSelector('#inspector-dialog[open]');
      assert.match(await page.locator('#inspect-output').textContent(), /preserving bg_opacity/);
      await page.locator('#inspector-dialog [value="close"]').click();

      await more('providers');
      await page.locator('#providers .provider-detail .primary').click();
      await page.waitForSelector('#connect[open]');
      await page.locator('#connect .tile').last().click();
      await page.locator('#wizard-next').click();
      await page.fill('#wizard-root', mockRoot);
      await page.fill('#wizard-name', 'Smoke gateway');
      await page.fill('#wizard-env', 'SMOKE_PROVIDER_KEY');
      await page.waitForSelector('.found.yes');
      await page.locator('#wizard-test').click();
      await page.waitForSelector('.model-row');
      await page.locator('.model-row', { hasText: 'smoke/fast' }).click();
      await page.locator('#wizard-save').click(); await idle();
      await page.waitForSelector('#connect', { state: 'hidden' });
      assert.equal(await page.locator('#model-name').textContent(), 'fast');
      const providers = (await snapshot()).providers;
      assert.deepEqual(providers.selected, { provider: 'smoke-gateway', model: 'smoke/fast' });
      assert.ok(!fs.readFileSync(providersFile, 'utf8').includes('smoke-key'));
      await page.fill('#request', 'Darker comments'); await page.keyboard.press('Enter'); await idle();
      current = await state();
      assert.ok(current.candidate, 'Expected a candidate from the mock provider');
      assert.ok(current.changes.every(change => change.role === 'syntax.comment'));
      await click('#reject');

      await page.fill('#request', 'HANG on this one'); await page.keyboard.press('Enter');
      await page.waitForSelector('.card.thinking');
      await page.waitForFunction(() => /Waiting for the model/.test(document.querySelector('#live-phase').textContent));
      assert.equal(await page.locator('#send').getAttribute('aria-label'), 'Stop the request');
      await page.locator('#live-cancel').click();
      await page.waitForFunction(() => /Stopped after/.test(document.querySelector('#timeline').textContent));
      assert.equal((await page.evaluate(async () => (await fetch('/api/live')).json())).active, false);
      assert.ok(!(await state()).candidate);

      await page.fill('#request', 'BADMODEL please'); await page.keyboard.press('Enter'); await idle();
      await page.waitForSelector('.card.failure');
      assert.match(await page.locator('.card.failure').textContent(), /refused the request/);
      await page.locator('.card.failure .ghost-button', { hasText: 'Try again' }).click(); await idle();
      assert.equal(await page.locator('.card.failure').count(), 2);
    }

    if (live) {
      async function prompt(request) {
        await page.fill('#request', request); await page.keyboard.press('Enter'); await idle();
        const result = await state();
        assert.ok(result.candidate, `Expected candidate for ${request}; last_result=${JSON.stringify(result.last_result)}; session=${await page.locator('#timeline').textContent()}`);
        return result;
      }
      current = await prompt('No italics anywhere.');
      for (const style of Object.values(current.candidate.theme.styles)) assert.equal(style.italic, false);
      for (const rule of current.candidate.theme.modifiers) assert.equal(rule.style.italic, false);
      assert.ok(current.changes.every(change => change.property === 'italic')); await click('#reject');
      current = await prompt('Keep the background color and make comments easier to read.');
      assert.equal(current.candidate.theme.styles['surface.base'].bg, current.current.styles['surface.base'].bg);
      assert.ok(current.candidate.validation.scenes.find(scene => scene.name === 'normal/syntax.comment').contrast >= 4.5);
      assert.ok(current.changes.every(change => (change.role === 'syntax.comment' && change.property === 'fg') || (change.role === 'surface.base' && change.property === 'bg' && change.locked_after)));
      await click('#accept');
      current = await prompt('Make the background darker.');
      assert.equal(current.candidate.theme.styles['surface.base'].bg_opacity, current.current.styles['surface.base'].bg_opacity);
      assert.ok(current.changes.every(change => change.role === 'surface.base' && change.property === 'bg')); await click('#reject');
      await page.fill('#request', 'Make that blue quieter.'); await page.keyboard.press('Enter'); await idle();
      current = await state();
      assert.ok(current.clarification, 'Ambiguous blue should request clarification.');
      await page.fill('#request', 'I mean the foreground color of syntax keywords (syntax.keyword fg). Make that color slightly less saturated; keep all opacity, typography, backgrounds, and other roles unchanged.');
      await page.keyboard.press('Enter'); await idle();
      current = await state();
      assert.ok(current.candidate, 'Live clarification should produce a candidate: ' + JSON.stringify({ clarification: current.clarification, last_result: current.last_result }));
      assert.ok(current.changes.every(change => change.role === 'syntax.keyword' && change.property === 'fg'), 'Clarification changed unrelated properties: ' + JSON.stringify({ changes: current.changes, spec: current.candidate.spec }));
      await click('#reject');
      console.log('Live: typography, readability/preservation, darker-with-opacity-preserved, clarification continuation passed.');
    }

    assert.deepEqual(errors, []);
    const screenshot = process.env.HEXWITCH_SCREENSHOT || path.join(__dirname, '../experiments/hexwitch-playground.png');
    fs.mkdirSync(path.dirname(screenshot), { recursive: true });
    await page.screenshot({ path: screenshot, fullPage: true });
    console.log('Browser: nudges, stacking, compare, keep/undo/redo, semantic italics, readability, matrix, themes, replay clarification, offline prompt' + (live ? '' : ', connect-a-model flow, live status, stop, failure cards') + ' passed.');
    await browser.close(); browser = null;
  } catch (error) {
    await exportFailure();
    throw error;
  } finally {
    if (browser) await browser.close();
    mock.close();
    if (host.exitCode === null && host.signalCode === null) { host.kill(); await once(host, 'exit'); }
    fs.rmSync(providersFile, { force: true });
  }
})().catch(error => { console.error(error.message); process.exitCode = 1; });
