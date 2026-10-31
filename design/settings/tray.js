// Draws the tray icon states, its menu and the reload-failure notification as
// they appear on Windows. The menu and the notification are drawn by the OS, so
// they follow the Windows light or dark setting rather than the mosaico theme.
//   node design/settings/tray.js <x> <y>
const path = require('path');
const { connect } = require(path.join(__dirname, 'fgsession.js'));
const { roles } = require('./theme.js');
const { makeUi } = require('./ui.js');

const LOGO = path.resolve(__dirname, '..', '..', 'assets', 'png', 'mosaico-64.png');
const W = { desktop: '#1F2430', taskbar: '#1C1C1C', taskbarLight: '#EEEEEE', menu: '#2C2C2C', border: '#3D3D3D', hover: '#FFFFFF14', text: '#FFFFFF', dim: '#C5C5C5', faint: '#9A9A9A', red: '#E81123', label: '#AEB4C2' };

async function main() {
    const [x = '2120', y = '0'] = process.argv.slice(2);
    const fg = connect();
    const ui = makeUi(fg, roles('mocha'));
    const name = 'Tray / Windows';
    const doc = await fg.call('inspect_document', {});
    const top = await fg.call('get_node', { id: doc.currentPage, depth: 1 });
    const old = (top.children || []).filter((n) => n.name === name).map((n) => n.id);
    if (old.length) await fg.call('run_command', { command: 'edit.delete', params: { ids: old } });

    const root = await ui.frame(undefined, name, { fill: W.desktop, cornerRadius: 8, clip: true }, { x: +x, y: +y, width: 980, height: 700 });
    const label = (parent, s, px, py) => ui.node('text', parent, { absolute: true, fontFamily: 'Inter', fontStyle: 'SemiBold', fontSize: 12, fill: W.label }, { text: s, x: px, y: py });
    const logo = async (parent, size, paused) => {
        const holder = await ui.frame(parent, paused ? 'icon/paused' : 'icon/running', { clip: false }, { width: size, height: size });
        const img = await ui.frame(holder, 'logo', { layout: 'horizontal', opacity: paused ? 0.45 : 1 }, { width: size, height: size });
        await ui.image(img, LOGO, size, 'logo');
        if (paused) {
            const badge = await ui.frame(holder, 'pause badge', { absolute: true, layout: 'horizontal', fill: W.red, cornerRadius: 6, mainAlign: 'center', crossAlign: 'center' }, { x: size - 8, y: size - 8, width: 12, height: 12 });
            await ui.icon(badge, 'pause', '#FFFFFF', 8, 2);
        }
        return holder;
    };

    // Icon variants
    await label(root, 'Icon: running and paused, on a dark and a light taskbar', 40, 36);
    const variants = [['Dark taskbar', W.taskbar], ['Light taskbar', W.taskbarLight]];
    for (const [i, [title, bg]] of variants.entries()) {
        for (const [j, paused] of [false, true].entries()) {
            const tile = await ui.frame(root, `variant/${title}/${paused ? 'paused' : 'running'}`, { absolute: true, layout: 'horizontal', fill: bg, cornerRadius: 8, mainAlign: 'center', crossAlign: 'center', stroke: '#FFFFFF1A', strokeWeight: 1 }, { x: 40 + j * 140, y: 64 + i * 104, width: 124, height: 64 });
            await logo(tile, 24, paused);
            await ui.node('text', root, { absolute: true, fontFamily: 'Inter', fontSize: 11, fill: W.faint }, { text: `${paused ? 'Paused' : 'Running'}, ${title.toLowerCase()}`, x: 40 + j * 140, y: 134 + i * 104 });
        }
    }

    // Tooltips
    await label(root, 'Tooltip', 40, 288);
    for (const [i, t] of ['mosaico 0.11.0', 'mosaico 0.11.0 (paused)'].entries()) {
        const tip = await ui.frame(root, `tooltip/${i}`, { absolute: true, layout: 'horizontal', fill: W.menu, stroke: W.border, strokeWeight: 1, cornerRadius: 6, padding: [6, 10, 6, 10], sizingH: 'hug', sizingV: 'hug' }, { x: 40, y: 312 + i * 44 });
        await ui.text(tip, t, 12, W.text);
    }

    // Reload failure notification
    await label(root, 'Notification when a reload fails', 560, 36);
    const toast = await ui.frame(root, 'notification', { absolute: true, layout: 'vertical', fill: '#2B2B2B', stroke: W.border, strokeWeight: 1, cornerRadius: 8, padding: 16, gap: 8, sizingV: 'hug', effects: [{ type: 'dropShadow', color: '#00000080', offset: [0, 8], radius: 24 }] }, { x: 560, y: 64, width: 380 });
    const th = await ui.hstack(toast, 'app', { gap: 8, sizingH: 'fill' });
    await ui.image(th, LOGO, 16, 'logo');
    await ui.text(th, 'mosaico', 12, W.dim);
    await ui.spacer(th);
    await ui.icon(th, 'close', W.faint, 12, 1.5);
    await ui.text(toast, 'Config not reloaded', 14, W.text, 'SemiBold');
    await ui.text(toast, 'bar.toml, line 12: unknown widget type "clok". mosaico keeps the previous settings.', 13, W.dim, 'Regular', { sizingH: 'fill', textAutoResize: 'height' });
    const tb = await ui.hstack(toast, 'buttons', { gap: 8, sizingH: 'fill', padding: [4, 0, 0, 0] });
    for (const b of ['Open settings', 'Open log']) {
        const f = await ui.frame(tb, `button/${b}`, { layout: 'horizontal', fill: '#3A3A3A', cornerRadius: 4, mainAlign: 'center', crossAlign: 'center', sizingH: 'fill' }, { height: 30 });
        await ui.text(f, b, 13, W.text);
    }

    // Taskbar with the tray and the menu open
    const bar = await ui.frame(root, 'taskbar', { absolute: true, layout: 'horizontal', fill: W.taskbar, padding: [0, 12, 0, 12], gap: 4, crossAlign: 'center', strokeTopWeight: 1, stroke: '#2E2E2E' }, { x: 0, y: 652, width: 980, height: 48 });
    await ui.spacer(bar);
    const trayItem = async (key, hovered) => {
        const f = await ui.frame(bar, `tray/${key}`, { layout: 'horizontal', fill: hovered ? W.hover : 'none', cornerRadius: 4, mainAlign: 'center', crossAlign: 'center' }, { width: 32, height: 40 });
        if (key === 'mosaico') await logo(f, 16, false);
        else await ui.icon(f, key, W.dim, 15, 1.5);
        return f;
    };
    await trayItem('up');
    const mosaico = await trayItem('mosaico', true);
    await trayItem('wifi');
    await trayItem('volume');
    const clock = await ui.vstack(bar, 'clock', { gap: 1, padding: [0, 8, 0, 8], crossAlign: 'max' });
    await ui.text(clock, '12:57', 12, W.text);
    await ui.text(clock, '31/10/2026', 12, W.text);

    const items = [['Settings...', true], '-', ['Pause tiling', false, true], ['Reload config'], '-', ['Open config folder'], ['Open log'], '-', ['Quit mosaico']];
    const b = await ui.boundsIn(root, mosaico);
    const menuH = items.filter((i) => i !== '-').length * 32 + items.filter((i) => i === '-').length * 9 + 10;
    const menu = await ui.frame(root, 'menu', { absolute: true, layout: 'vertical', fill: W.menu, stroke: W.border, strokeWeight: 1, cornerRadius: 8, padding: 5, sizingV: 'hug', effects: [{ type: 'dropShadow', color: '#00000099', offset: [0, 8], radius: 24 }] }, { x: b.x + b.width - 236, y: b.y - 8 - menuH, width: 236 });
    for (const it of items) {
        if (it === '-') { const s = await ui.frame(menu, 'separator', { layout: 'vertical', padding: [4, 8, 4, 8], sizingH: 'fill', sizingV: 'hug' }); await ui.node('rectangle', s, { fill: W.border, sizingH: 'fill' }, { width: 10, height: 1, name: 'line' }); continue; }
        const [t, bold, hover] = it;
        const f = await ui.frame(menu, `item/${t}`, { layout: 'horizontal', fill: hover ? W.hover : 'none', cornerRadius: 4, padding: [0, 12, 0, 12], crossAlign: 'center', sizingH: 'fill' }, { height: 32 });
        await ui.text(f, t.replace('...', '\u2026'), 13, W.text, bold ? 'SemiBold' : 'Regular');
    }
    await label(root, 'Menu: right-click (left click opens Settings)', b.x + b.width - 236, b.y - 8 - menuH - 26);

    console.log(JSON.stringify({ window: root, name }));
    fg.close();
}

main().catch((e) => { console.error(e.message); process.exit(1); });
