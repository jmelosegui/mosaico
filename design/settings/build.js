// Draws one settings window into the open FigCraft document, replacing any
// window with the same name.
//   node design/settings/build.js <page> <flavor> <x> <y>
// <page> is a key of PAGES in pages.js, such as "General" or "Bar widgets".
const path = require('path');
const { connect } = require(path.join(__dirname, 'fgsession.js'));
const { roles } = require('./theme.js');
const { makeUi } = require('./ui.js');
const { PAGES } = require('./pages.js');

const LOGO = path.resolve(__dirname, '..', '..', 'assets', 'png', 'mosaico-64.png');
const NAV = ['General', 'Layout', 'Workspaces', 'Borders', 'Mouse', 'Animations', 'Bar', 'Keybindings', 'Rules'];
const FILES = { Bar: 'bar.toml', Keybindings: 'keybindings.toml', Rules: 'user-rules.toml and rules.toml' };

async function main() {
    const [page = 'General', flavor = 'mocha', x = '0', y = '0'] = process.argv.slice(2);
    if (!PAGES[page]) throw new Error(`unknown page ${page}; use one of: ${Object.keys(PAGES).join(', ')}`);
    const c = roles(flavor);
    const nav = NAV.find((p) => page === p || page.startsWith(p + ' '));
    const fg = connect();
    const ui = makeUi(fg, c);
    const name = `Settings / ${page} / ${c.name}`;

    const doc = await fg.call('inspect_document', {});
    const top = await fg.call('get_node', { id: doc.currentPage, depth: 1 });
    const old = (top.children || []).filter((n) => n.name === name).map((n) => n.id);
    if (old.length) await fg.call('run_command', { command: 'edit.delete', params: { ids: old } });

    const win = await ui.frame(undefined, name, { layout: 'vertical', fill: c.base, cornerRadius: 8, clip: true, stroke: c.crust, strokeWeight: 1, effects: [{ type: 'dropShadow', color: '#00000066', offset: [0, 8], radius: 32 }] }, { x: +x, y: +y, width: 980, height: 700 });

    const title = await ui.frame(win, 'titlebar', { layout: 'horizontal', fill: c.crust, padding: [0, 0, 0, 12], gap: 8, crossAlign: 'center', sizingH: 'fill' }, { height: 36 });
    await ui.image(title, LOGO, 16, 'app icon');
    await ui.text(title, 'mosaico settings', 12, c.subtext0);
    await ui.spacer(title);
    for (const k of ['minimize', 'maximize', 'close']) {
        const b = await ui.frame(title, `window/${k}`, { layout: 'horizontal', mainAlign: 'center', crossAlign: 'center' }, { width: 46, height: 36 });
        await ui.icon(b, k, c.subtext0, 14, 1.25);
    }

    const body = await ui.frame(win, 'body', { layout: 'horizontal', sizingH: 'fill', sizingV: 'fill' });

    const side = await ui.frame(body, 'sidebar', { layout: 'vertical', fill: c.mantle, padding: [16, 8, 12, 8], gap: 2, sizingV: 'fill', strokeRightWeight: 1, stroke: c.surface0 }, { width: 224 });
    const search = await ui.frame(side, 'search', { layout: 'horizontal', fill: c.base, cornerRadius: 5, padding: [0, 10, 0, 10], gap: 8, crossAlign: 'center', sizingH: 'fill' }, { height: 28 });
    await ui.icon(search, 'search', c.overlay0, 14);
    await ui.text(search, 'Search settings', 12, c.overlay0);
    await ui.frame(side, 'gap', {}, { width: 1, height: 10 });
    for (const p of NAV) {
        const on = p === nav;
        const item = await ui.frame(side, `nav/${p}`, { layout: 'horizontal', fill: on ? c.selected : 'none', cornerRadius: 6, padding: [0, 10, 0, 10], gap: 10, crossAlign: 'center', sizingH: 'fill' }, { height: 32 });
        await ui.icon(item, p.toLowerCase(), on ? c.accent : c.subtext0, 16);
        await ui.text(item, p, 13, on ? c.text : c.subtext0, on ? 'SemiBold' : 'Regular');
    }
    await ui.spacer(side);
    const status = await ui.frame(side, 'daemon status', { layout: 'horizontal', gap: 10, padding: [8, 10, 4, 10], crossAlign: 'center', sizingH: 'fill', sizingV: 'hug' });
    await ui.node('ellipse', status, { fill: c.green }, { width: 8, height: 8, name: 'running' });
    const st = await ui.vstack(status, 'text', { gap: 1 });
    await ui.text(st, 'Daemon running', 12, c.text);
    await ui.text(st, 'mosaico 0.11.0', 11, c.overlay0);

    const content = await ui.frame(body, 'content', { layout: 'vertical', sizingH: 'fill', sizingV: 'fill', fill: c.page });
    const scroll = await ui.frame(content, 'page', { layout: 'vertical', padding: [24, 32, 24, 32], gap: 20, sizingH: 'fill', sizingV: 'fill', clip: true });
    const header = await ui.frame(scroll, 'header', { layout: 'vertical', gap: 2, sizingH: 'fill', sizingV: 'hug' });
    await ui.text(header, nav, 18, c.text, 'SemiBold');
    await ui.text(header, FILES[nav] || 'config.toml', 11, c.overlay0);

    const result = await PAGES[page](ui, c, scroll, fg, win, header);

    const footer = await ui.frame(content, 'footer', { layout: 'horizontal', fill: c.mantle, padding: [0, 24, 0, 24], gap: 8, crossAlign: 'center', sizingH: 'fill', strokeTopWeight: 1, stroke: c.surface0 }, { height: 52 });
    const n = result.unsaved || 0;
    if (n) await ui.node('ellipse', footer, { fill: c.accent }, { width: 6, height: 6, name: 'unsaved' });
    await ui.text(footer, n ? `${n} unsaved change${n > 1 ? 's' : ''}` : 'No unsaved changes', 12, c.subtext0);
    await ui.spacer(footer);
    await ui.button(footer, 'Revert', false, { disabled: !n });
    await ui.button(footer, 'Apply', true, { disabled: !n });

    if (result.scrolls) {
        const b = await ui.boundsIn(content, scroll);
        await ui.node('rectangle', content, { absolute: true, fill: c.surface1, cornerRadius: 2 }, { x: b.width - 8, y: 8, width: 4, height: Math.round(b.height * 0.6), name: 'scrollbar' });
    }
    if (result.after) await result.after();

    console.log(JSON.stringify({ window: win, name }));
    fg.close();
}

main().catch((e) => { console.error(e.message); process.exit(1); });
