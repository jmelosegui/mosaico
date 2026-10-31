// Page contents. Each builder fills the scrolling page and returns
// { unsaved, scrolls, after }: the number of unsaved changes, whether the page
// is taller than the window, and a step that runs once the window is laid out
// (for popovers that are positioned over other layers).
const LAYOUT_TILES = {
    bsp: [[0, 0, 0.5, 1], [0.5, 0, 0.5, 0.5], [0.5, 0.5, 0.25, 0.5], [0.75, 0.5, 0.25, 0.5]],
    'vertical-stack': [[0, 0, 0.5, 1], [0.5, 0, 0.5, 1 / 3], [0.5, 1 / 3, 0.5, 1 / 3], [0.5, 2 / 3, 0.5, 1 / 3]],
    'three-column': [[0.25, 0, 0.5, 1], [0, 0, 0.25, 0.5], [0, 0.5, 0.25, 0.5], [0.75, 0, 0.25, 1]],
};
const LAYOUT_NAMES = { bsp: 'BSP', 'vertical-stack': 'Vertical stack', 'three-column': 'Three column' };

// The status bar as it is drawn, at half size, with the flavor's bar colors.
async function barPreview(ui, c, parent) {
    const b = c.bar;
    const desk = await ui.frame(parent, 'bar preview', { layout: 'vertical', fill: c.desktop, cornerRadius: 8, padding: 10, sizingH: 'fill', sizingV: 'hug', stroke: c.line, strokeWeight: 1 });
    const bar = await ui.frame(desk, 'bar', { layout: 'horizontal', fill: b.background, padding: [0, 4, 0, 4], gap: 5, crossAlign: 'center', sizingH: 'fill' }, { height: 32 });
    const pill = async (label, o = {}) => {
        const p = await ui.hstack(bar, `pill/${label || o.icon}`, { fill: o.fill || b.widgetBackground, stroke: o.border || b.pillBorder, strokeWeight: 1, cornerRadius: 2, padding: [3, 6, 3, 6], gap: 4 });
        if (o.icon) await ui.icon(p, o.icon, o.color || b.foreground, 11, 2);
        if (label) await ui.text(p, label, 11, o.color || b.foreground, 'Bold');
        return p;
    };
    const ws = await ui.hstack(bar, 'workspaces', { gap: 2 });
    for (const n of [1, 2, 3, 4]) {
        const on = n === 1;
        const p = await ui.hstack(ws, `workspace/${n}`, { fill: on ? b.activeWorkspace : b.widgetBackground, stroke: b.pillBorder, strokeWeight: 1, cornerRadius: 2, padding: [3, 7, 3, 7] });
        await ui.text(p, String(n), 11, on ? b.activeWorkspaceText : b.inactiveWorkspace, 'Bold');
    }
    await pill('mosaico settings', { icon: 'window' });
    await pill('BSP', { icon: 'grid' });
    await ui.spacer(bar);
    await pill('12%', { icon: 'cpu' });
    await pill('48%', { icon: 'ram' });
    await pill('Saturday 31 October 2026', { icon: 'calendar' });
    await pill('12:57:03', { icon: 'clock' });
    return desk;
}

const PAGES = {
    async General(ui, c, scroll) {
        let card = await ui.section(scroll, 'Appearance');
        await ui.row(card, 'Theme', 'Colors for window borders, the bar and this window.', (r) => ui.dropdown(r, 'Catppuccin'));
        await ui.row(card, 'Flavor', null, (r) => ui.segmented(r, ['Mocha', 'Macchiato', 'Frappe', 'Latte'], c.name), { last: true });
        card = await ui.section(scroll, 'Startup');
        await ui.row(card, 'Tray icon', 'Show the mosaico icon in the notification area while the daemon runs.', (r) => ui.toggle(r, true));
        await ui.row(card, 'Start at sign-in', 'Start the daemon when you sign in.', (r) => ui.toggle(r, false), { last: true });
        card = await ui.section(scroll, 'Logging');
        await ui.row(card, 'Write a log file', 'Logs go to ~/.config/mosaico/logs/mosaico.log.', (r) => ui.toggle(r, true), { changed: true, def: 'off' });
        await ui.row(card, 'Level', 'The least severe messages that are written.', (r) => ui.dropdown(r, 'info', 120));
        await ui.row(card, 'Maximum file size', 'The log is rotated when it reaches this size.', (r) => ui.spin(r, '10', 'MB'), { last: true });
        return { unsaved: 1 };
    },

    async Layout(ui, c, scroll, fg) {
        const s = await ui.frame(scroll, 'section/Default layout', { layout: 'vertical', gap: 8, sizingH: 'fill', sizingV: 'hug' });
        await ui.text(s, 'Default layout', 13, c.subtext0, 'SemiBold');
        const cards = await ui.frame(s, 'layout picker', { layout: 'horizontal', gap: 12, sizingH: 'fill', sizingV: 'hug' });
        const previews = [];
        for (const [k, rects] of Object.entries(LAYOUT_TILES)) {
            const on = k === 'bsp';
            const card = await ui.frame(cards, `layout/${k}`, { layout: 'vertical', gap: 10, padding: 12, fill: c.card, cornerRadius: 8, stroke: on ? c.accent : c.line, strokeWeight: on ? 2 : 1, sizingH: 'fill', sizingV: 'hug' });
            const pv = await ui.frame(card, 'preview', { layout: 'horizontal', fill: c.control, cornerRadius: 5, sizingH: 'fill' }, { height: 96 });
            previews.push([pv, rects, on]);
            await ui.text(card, LAYOUT_NAMES[k], 13, c.text, on ? 'SemiBold' : 'Regular');
        }
        for (const [pv, rects, on] of previews) {
            const W = (await fg.call('get_node', { id: pv, depth: 0 })).width, H = 96, g = 3, p = 4;
            for (const [i, [rx, ry, rw, rh]] of rects.entries()) {
                await ui.node('rectangle', pv, { absolute: true, fill: i === 0 && on ? c.accent : c.surface1, cornerRadius: 3 }, { x: Math.round(p + rx * (W - 2 * p) + g), y: Math.round(p + ry * (H - 2 * p) + g), width: Math.round(rw * (W - 2 * p) - 2 * g), height: Math.round(rh * (H - 2 * p) - 2 * g), name: `tile ${i + 1}` });
            }
        }
        let card = await ui.section(scroll, 'Spacing');
        await ui.row(card, 'Gap', 'Pixels between windows and the screen edges.', (r) => ui.slider(r, 12 / 48, '12 px'), { changed: true, def: '8 px' });
        await ui.row(card, 'Split ratio', 'Share of each split given to the first window.', (r) => ui.slider(r, 0.5, '0.50'), { last: true });
        card = await ui.section(scroll, 'Workspace switching');
        await ui.row(card, 'Hide windows on other workspaces', 'How windows on workspaces you are not looking at are put away. The list shows the ways this system supports.', (r) => ui.dropdown(r, 'Keep in taskbar (recommended)', 240), { last: true });
        return { unsaved: 1 };
    },

    async Workspaces(ui, c, scroll) {
        let card = await ui.section(scroll, 'Switching');
        await ui.row(card, 'Mode', 'Per monitor switches only the focused monitor. Global switches every monitor together.', (r) => ui.segmented(r, ['Per monitor', 'Global'], 'Per monitor'));
        await ui.row(card, 'Wrap around', 'Stepping past the last workspace goes back to the first, and before the first to the last.', (r) => ui.toggle(r, false), { last: true });
        card = await ui.section(scroll, 'Layout per workspace', { help: 'Workspaces without a layout of their own use the default layout from the Layout page.' });
        const values = { 1: 'Vertical stack', 3: 'Three column' };
        for (let r = 0; r < 4; r++) {
            const line = await ui.frame(card, `line/${r}`, { layout: 'horizontal', sizingH: 'fill', sizingV: 'hug', ...(r < 3 ? { strokeBottomWeight: 1, stroke: c.line } : {}) });
            for (const k of [0, 1]) {
                const n = k * 4 + r + 1;
                const cell = await ui.frame(line, `workspace/${n}`, { layout: 'horizontal', padding: [9, 16, 9, 16], gap: 12, crossAlign: 'center', sizingH: 'fill', sizingV: 'hug', ...(k === 0 ? { strokeRightWeight: 1, stroke: c.line } : {}) });
                const badge = await ui.frame(cell, 'number', { layout: 'horizontal', fill: values[n] ? c.selected : c.control, cornerRadius: 5, mainAlign: 'center', crossAlign: 'center' }, { width: 24, height: 24 });
                await ui.text(badge, String(n), 12, values[n] ? c.accent : c.subtext0, 'SemiBold');
                await ui.text(cell, `Workspace ${n}`, 13, c.text, 'Regular', { sizingH: 'fill' });
                await ui.dropdown(cell, values[n] || 'Default (BSP)', 168, { dim: !values[n], changed: n === 3 });
            }
        }
        return { unsaved: 1 };
    },

    async Borders(ui, c, scroll, fg, win) {
        let card = await ui.section(scroll, 'Border');
        await ui.row(card, 'Width', 'Border width around the focused window.', (r) => ui.slider(r, 4 / 16, '4 px'));
        await ui.row(card, 'Corner style', 'Corners of borders and tiled windows.', (r) => ui.segmented(r, ['Square', 'Small', 'Round'], 'Small'), { last: true });
        card = await ui.section(scroll, 'Colors', { help: 'A theme color follows the theme when you switch flavors. A hex color stays fixed.' });
        let focused;
        await ui.row(card, 'Focused', 'While a tiled layout is active.', async (r) => { focused = await ui.dropdown(r, 'mauve', 190, { swatch: c.accents[3][1], open: true }); }, { changed: true, def: 'blue' });
        await ui.row(card, 'Monocle', 'While monocle mode is active.', (r) => ui.dropdown(r, 'Theme default (green)', 190, { swatch: c.green, dim: true }));
        await ui.row(card, 'Unfocused', 'Around unfocused tiled windows. None turns them off.', (r) => ui.dropdown(r, 'None', 190, { swatch: 'none' }), { last: true });
        return {
            unsaved: 1,
            after: async () => {
                const b = await ui.boundsIn(win, focused);
                const pop = await ui.popover(win, b.x + b.width - 254, b.y + b.height + 6, 254, 'color picker');
                await ui.text(pop, 'Theme colors', 11, c.subtext0, 'SemiBold');
                const grid = await ui.vstack(pop, 'swatches', { gap: 6 });
                for (const half of [c.accents.slice(0, 7), c.accents.slice(7)]) {
                    const line = await ui.hstack(grid, 'line', { gap: 6 });
                    for (const [name, hex] of half) {
                        const on = name === 'mauve';
                        await ui.node('rectangle', line, { fill: hex, cornerRadius: 6, ...(on ? { stroke: c.text, strokeWeight: 2 } : {}) }, { width: 26, height: 26, name: `swatch/${name}` });
                    }
                }
                const sel = await ui.hstack(pop, 'selected', { gap: 8, sizingH: 'fill' });
                await ui.text(sel, 'mauve', 12, c.text, 'SemiBold');
                await ui.spacer(sel);
                await ui.text(sel, c.accents[3][1], 12, c.overlay1);
                await ui.divider(pop);
                const custom = await ui.hstack(pop, 'custom', { gap: 8, sizingH: 'fill' });
                await ui.text(custom, 'Hex', 12, c.subtext0, 'Regular', { sizingH: 'fill' });
                await ui.textField(custom, '', 120, { placeholder: '#rrggbb' });
                const reset = await ui.hstack(pop, 'theme default', { gap: 8, sizingH: 'fill' });
                await ui.icon(reset, 'reset', c.accent, 13);
                await ui.text(reset, 'Use the theme default (blue)', 12, c.accent);
            },
        };
    },

    async Mouse(ui, c, scroll) {
        let card = await ui.section(scroll, 'Focus');
        await ui.row(card, 'Mouse follows focus', 'Move the cursor to the center of the focused window when you navigate with the keyboard.', (r) => ui.toggle(r, true));
        await ui.row(card, 'Focus follows mouse', 'Focus the window under the cursor without clicking.', (r) => ui.toggle(r, false), { last: true });
        card = await ui.section(scroll, 'Dragging');
        await ui.row(card, 'Drag to swap', 'Drop a tiled window you drag by its title bar on another tile to swap the two, or on another monitor to put it before the tile under the pointer. Off: the window goes back to its slot.', (r) => ui.toggle(r, true));
        await ui.row(card, 'Resize splits', 'Drag the edge between two tiles to move the split between them. A corner where two splits meet moves both. Off: a resized window goes back to its slot.', (r) => ui.toggle(r, true), { last: true });
        return { unsaved: 0 };
    },

    async Animations(ui, c, scroll) {
        let card = await ui.section(scroll, 'Transitions');
        await ui.row(card, 'Animate windows', 'Windows move to their new place over a short time when they swap, move or resize, and when the layout changes.', (r) => ui.toggle(r, true));
        await ui.row(card, 'Duration', 'How long a transition takes.', (r) => ui.slider(r, 180 / 1000, '180 ms'));
        await ui.row(card, 'Easing', 'How windows speed up and slow down on the way.', (r) => ui.dropdown(r, 'Ease out (cubic)', 190), { last: true });
        const curveRow = await ui.frame(card, 'curve', { layout: 'horizontal', padding: [0, 16, 16, 16], gap: 16, sizingH: 'fill', sizingV: 'hug' });
        const box = await ui.frame(curveRow, 'curve preview', { fill: c.control, cornerRadius: 6 }, { width: 232, height: 104 });
        const W = 200, H = 72, pts = [];
        for (let i = 0; i <= 32; i++) { const t = i / 32; pts.push(`${(t * W).toFixed(1)} ${(H - (1 - Math.pow(1 - t, 3)) * H).toFixed(1)}`); }
        await ui.node('vector', box, { absolute: true, fill: 'none', stroke: c.surface1, strokeWeight: 1 }, { path: `M0 ${H}L${W} 0`, x: 16, y: 16, width: W, name: 'linear' });
        await ui.node('vector', box, { absolute: true, fill: 'none', stroke: c.accent, strokeWeight: 2, strokeCap: 'round', strokeJoin: 'round' }, { path: 'M' + pts.join('L'), x: 16, y: 16, width: W, name: 'ease-out-cubic' });
        await ui.node('ellipse', box, { absolute: true, fill: c.accent }, { x: 13, y: 16 + H - 3, width: 6, height: 6, name: 'start' });
        await ui.node('ellipse', box, { absolute: true, fill: c.accent }, { x: 16 + W - 3, y: 13, width: 6, height: 6, name: 'end' });
        const side = await ui.vstack(curveRow, 'custom', { gap: 8, sizingH: 'fill' });
        await ui.text(side, 'Custom curve', 12, c.subtext0, 'SemiBold');
        await ui.text(side, 'Choose Custom to enter cubic-bezier(x1, y1, x2, y2), each from 0 to 1.', 11, c.subtext0, 'Regular', { sizingH: 'fill', textAutoResize: 'height' });
        const nums = await ui.hstack(side, 'points', { gap: 6, opacity: 0.45 });
        for (const v of ['0.33', '1.00', '0.68', '1.00']) await ui.spin(nums, v, null, { width: 56 });
        await ui.button(side, 'Play', false, { icon: 'animations', small: true, outline: true });
        card = await ui.section(scroll, 'System');
        await ui.row(card, 'Follow the system setting', 'Snap windows into place while the system has animations turned off, and in a remote session.', (r) => ui.toggle(r, true), { last: true });
        return { unsaved: 0 };
    },

    async Bar(ui, c, scroll, fg, win, header) {
        await ui.tabs(header, ['Appearance', 'Widgets', 'Mouse'], 'Appearance');
        await barPreview(ui, c, scroll);
        let card = await ui.section(scroll, 'Bar');
        await ui.row(card, 'Show the bar', null, (r) => ui.toggle(r, true));
        await ui.row(card, 'Monitors', 'With none selected, the bar shows on every monitor.', async (r) => { const h = await ui.hstack(r, 'monitors', { gap: 14 }); await ui.checkbox(h, true, 'Monitor 1'); await ui.checkbox(h, false, 'Monitor 2'); }, { changed: true, def: 'all' });
        await ui.row(card, 'Height', 'Increase for high-DPI displays.', (r) => ui.spin(r, '64', 'px'));
        await ui.row(card, 'Background opacity', null, (r) => ui.slider(r, 0, '0%'), { last: true });
        card = await ui.section(scroll, 'Font');
        await ui.row(card, 'Family', 'Installed fonts. Widget icons need a Nerd Font.', (r) => ui.dropdown(r, 'CaskaydiaCove Nerd Font', 240));
        await ui.row(card, 'Size', null, (r) => ui.spin(r, '24', 'px'));
        await ui.row(card, 'Style', null, async (r) => {
            const f = await ui.hstack(r, 'style', { fill: c.control, cornerRadius: 6, padding: 2, gap: 2 });
            for (const [label, style, on] of [['B', 'Bold', true], ['I', 'Italic', false], ['U', 'Regular', false]]) {
                const b = await ui.frame(f, `style/${label}`, { layout: 'horizontal', fill: on ? c.surface1 : 'none', cornerRadius: 4, mainAlign: 'center', crossAlign: 'center' }, { width: 28, height: 22 });
                await ui.text(b, label, 13, on ? c.text : c.subtext0, style, label === 'U' ? { textDecoration: 'underline' } : {});
            }
        }, { last: true });
        card = await ui.section(scroll, 'Pills and spacing');
        const grid = await ui.frame(card, 'spacing grid', { layout: 'vertical', padding: [12, 16, 14, 16], gap: 12, sizingH: 'fill', sizingV: 'hug' });
        for (const line of [[['Edge padding', '8'], ['Pill padding', '12'], ['Pill radius', '4']], [['Pill border', '2'], ['Item gap', '10'], ['Workspace gap', '4']]]) {
            const l = await ui.hstack(grid, 'line', { gap: 16, sizingH: 'fill' });
            for (const [label, v] of line) {
                const f = await ui.vstack(l, `field/${label}`, { gap: 5, sizingH: 'fill' });
                await ui.text(f, label, 11, c.subtext0);
                await ui.spin(f, v, 'px', { width: 140 });
            }
        }
        return { unsaved: 1, scrolls: true };
    },

    async 'Bar widgets'(ui, c, scroll, fg, win, header) {
        await ui.tabs(header, ['Appearance', 'Widgets', 'Mouse'], 'Widgets');
        await barPreview(ui, c, scroll);
        const cols = await ui.frame(scroll, 'columns', { layout: 'horizontal', gap: 16, sizingH: 'fill', sizingV: 'hug' });
        const list = async (title, note, items) => {
            const col = await ui.vstack(cols, `column/${title}`, { gap: 8, sizingH: 'fill' });
            const head = await ui.hstack(col, 'heading', { gap: 8, sizingH: 'fill' });
            await ui.text(head, title, 13, c.subtext0, 'SemiBold');
            await ui.text(head, note, 11, c.overlay0);
            const card = await ui.frame(col, 'card', { layout: 'vertical', fill: c.card, stroke: c.line, strokeWeight: 1, cornerRadius: 8, sizingH: 'fill', sizingV: 'hug', clip: false });
            for (const [i, it] of items.entries()) {
                const lifted = it.state === 'dragging';
                const selected = it.state === 'selected';
                if (it.state === 'drop') { await ui.node('rectangle', card, { fill: c.accent, cornerRadius: 1, sizingH: 'fill' }, { width: 10, height: 2, name: 'drop here' }); continue; }
                const r = await ui.frame(card, `widget/${it.name}`, { layout: 'horizontal', padding: [7, 12, 7, 6], gap: 8, crossAlign: 'center', sizingH: 'fill', sizingV: 'hug', fill: selected ? c.selected : lifted ? c.surface1 : 'none', ...(lifted ? { stroke: c.accent, strokeWeight: 1, cornerRadius: 6, effects: [{ type: 'dropShadow', color: '#00000055', offset: [0, 4], radius: 12 }] } : i < items.length - 1 && !selected ? { strokeBottomWeight: 1, stroke: c.line } : {}) });
                await ui.icon(r, 'grip', c.overlay0, 14, 2.5);
                await ui.icon(r, it.icon, it.enabled === false ? c.overlay0 : c.accent, 15);
                const lab = await ui.vstack(r, 'label', { gap: 1, sizingH: 'fill' });
                await ui.text(lab, it.name, 13, it.enabled === false ? c.overlay1 : c.text, selected ? 'SemiBold' : 'Regular');
                if (it.note) await ui.text(lab, it.note, 11, c.overlay0);
                await ui.toggle(r, it.enabled !== false);
                if (selected) {
                    const d = await ui.frame(card, 'clock options', { layout: 'vertical', padding: [6, 14, 12, 42], gap: 8, sizingH: 'fill', sizingV: 'hug', fill: c.selected, strokeBottomWeight: 1, stroke: c.line });
                    for (const [label, ctrl] of [['Format', (p) => ui.textField(p, '%H:%M:%S', 190, { hint: '12:57:03', focused: true })], ['Icon', (p) => ui.dropdown(p, 'Clock', 190, { icon: 'clock' })], ['Color', (p) => ui.dropdown(p, 'Theme default', 190, { swatch: c.bar.foreground, dim: true })]]) {
                        const l = await ui.hstack(d, `option/${label}`, { gap: 8, sizingH: 'fill' });
                        await ui.text(l, label, 12, c.subtext0, 'Regular', { sizingH: 'fill' });
                        await ctrl(l);
                    }
                }
            }
            await ui.button(col, 'Add widget', false, { icon: 'plus', small: true, ghost: true, color: c.accent });
        };
        await list('Left', 'from the left edge', [
            { name: 'Workspaces', icon: 'grid' },
            { name: 'Layout', icon: 'layout', state: 'dragging' },
            { state: 'drop' },
            { name: 'Active window', icon: 'window' },
        ]);
        await list('Right', 'from the right edge', [
            { name: 'Clock', icon: 'clock', state: 'selected' },
            { name: 'Date', icon: 'calendar', note: '%A %d %B %Y' },
            { name: 'RAM', icon: 'ram' },
            { name: 'CPU', icon: 'cpu' },
            { name: 'Update', icon: 'download', note: 'Only when an update is available' },
            { name: 'Paused', icon: 'pause', note: 'Only while paused' },
            { name: 'Media', icon: 'music', enabled: false, note: 'Only while something plays' },
        ]);
        return { unsaved: 1, scrolls: true };
    },

    async 'Bar mouse'(ui, c, scroll, fg, win, header) {
        await ui.tabs(header, ['Appearance', 'Widgets', 'Mouse'], 'Mouse');
        let card = await ui.section(scroll, 'Pointer');
        await ui.row(card, 'Take clicks and the wheel', 'Off lets the pointer through to the windows under the bar.', (r) => ui.toggle(r, true));
        await ui.row(card, 'Scrolling', 'What the wheel does over the bar.', (r) => ui.dropdown(r, 'Step through workspaces', 210), { last: true });
        card = await ui.section(scroll, 'Click actions', { help: 'A click on a workspace number always goes to that workspace. Workspace and layout actions act on the bar\'s own monitor.' });
        const cols = [['Widget', 170], ['Button', 120], ['Action', 0]];
        const head = await ui.frame(card, 'header', { layout: 'horizontal', padding: [8, 16, 8, 16], gap: 12, sizingH: 'fill', sizingV: 'hug', fill: c.control, strokeBottomWeight: 1, stroke: c.line });
        for (const [label, w] of cols) await ui.text(head, label, 11, c.subtext0, 'SemiBold', w ? { sizingH: 'fixed', width: w } : { sizingH: 'fill' });
        await ui.frame(head, 'actions', {}, { width: 26, height: 1 });
        const rows = [['Layout', 'Left', 'Cycle layout'], ['Layout', 'Right', 'Toggle monocle'], ['Paused', 'Left', 'Toggle pause'], ['Workspaces', 'Middle', 'Retile', true]];
        for (const [i, [w, b, a, added]] of rows.entries()) {
            const r = await ui.frame(card, `click/${w}/${b}`, { layout: 'horizontal', padding: [8, 16, 8, 16], gap: 12, crossAlign: 'center', sizingH: 'fill', sizingV: 'hug', ...(i < rows.length - 1 ? { strokeBottomWeight: 1, stroke: c.line } : {}) });
            await ui.dropdown(r, w, 170, { changed: added });
            await ui.dropdown(r, b, 120, { changed: added });
            await ui.dropdown(r, a, 200, { changed: added });
            await ui.node('frame', r, { fill: 'none', sizingH: 'fill' }, { width: 1, height: 1, name: 'spacer' });
            await ui.iconButton(r, 'trash', c.subtext0);
        }
        await ui.button(scroll, 'Add click action', false, { icon: 'plus', small: true, ghost: true, color: c.accent });
        return { unsaved: 1 };
    },

    async Keybindings(ui, c, scroll, fg, win, header) {
        const tools = await ui.hstack(header, 'tools', { gap: 8, sizingH: 'fill', padding: [12, 0, 0, 0] });
        await ui.textField(tools, '', 260, { placeholder: 'Filter by action or key', icon: 'search' });
        await ui.spacer(tools);
        await ui.button(tools, 'Restore defaults', false, { small: true, outline: true, icon: 'reset' });
        const group = async (title, items) => {
            const card = await ui.section(scroll, title);
            for (const [i, [label, id, ks, state]] of items.entries()) {
                await ui.row(card, label, null, async (r) => {
                    if (state === 'capture') {
                        await ui.text(r, 'Press the new shortcut', 11, c.accent);
                        await ui.keys(r, ks, { capturing: true });
                    } else {
                        await ui.keys(r, ks);
                    }
                    await ui.iconButton(r, 'edit', state === 'capture' ? c.accent : c.overlay0);
                }, { compact: true, sub: id, last: i === items.length - 1 && state !== 'capture', fill: state === 'capture' ? c.selected : undefined });
                if (state === 'capture') {
                    const warn = await ui.frame(card, 'conflict', { layout: 'horizontal', padding: [8, 16, 8, 16], gap: 10, crossAlign: 'center', sizingH: 'fill', sizingV: 'hug', fill: c.yellow + '1F', ...(i < items.length - 1 ? { strokeBottomWeight: 1, stroke: c.line } : {}) });
                    await ui.icon(warn, 'warning', c.yellow, 15);
                    const t = await ui.hstack(warn, 'message', { gap: 4, sizingH: 'fill' });
                    await ui.text(t, 'Alt Shift H is already bound to', 12, c.text);
                    await ui.text(t, 'Move left', 12, c.text, 'SemiBold');
                    await ui.button(warn, 'Cancel', false, { small: true, ghost: true });
                    await ui.button(warn, 'Replace', true, { small: true });
                }
            }
        };
        await group('Focus', [['Focus left', 'focus-left', ['Alt', 'Shift', 'H'], 'capture'], ['Focus down', 'focus-down', ['Alt', 'J']], ['Focus up', 'focus-up', ['Alt', 'K']], ['Focus right', 'focus-right', ['Alt', 'L']]]);
        await group('Move', [['Move left', 'move-left', ['Alt', 'Shift', 'H']], ['Move down', 'move-down', ['Alt', 'Shift', 'J']], ['Move up', 'move-up', ['Alt', 'Shift', 'K']], ['Move right', 'move-right', ['Alt', 'Shift', 'L']]]);
        await group('Windows and layout', [['Toggle monocle', 'toggle-monocle', ['Alt', 'T']], ['Cycle layout', 'cycle-layout', ['Alt', 'N']], ['Retile', 'retile', ['Alt', 'Shift', 'R']]]);
        return { unsaved: 0, scrolls: true };
    },

    async Rules(ui, c, scroll, fg, win, header) {
        const tools = await ui.hstack(header, 'tools', { gap: 8, sizingH: 'fill', padding: [12, 0, 0, 0] });
        await ui.text(tools, 'Rules are checked from the top and the first match wins. A window no rule matches is tiled.', 12, c.subtext0, 'Regular', { sizingH: 'fill', textAutoResize: 'height' });
        await ui.button(tools, 'Add rule', true, { small: true, icon: 'plus' });
        const decision = (r, tile) => ui.chip(r, tile ? 'Tile' : "Don't tile", { fill: (tile ? c.green : c.overlay0) + '26', color: tile ? c.green : c.subtext0, strong: true });
        const rule = async (card, matches, tile, o = {}) => {
            const r = await ui.frame(card, `rule/${matches.map((m) => m[1]).join(' ')}`, { layout: 'horizontal', padding: [9, 12, 9, o.readOnly ? 16 : 8], gap: 10, crossAlign: 'center', sizingH: 'fill', sizingV: 'hug', ...(o.last ? {} : { strokeBottomWeight: 1, stroke: c.line }), ...(o.editing ? { fill: c.selected } : {}) });
            if (!o.readOnly) await ui.icon(r, 'grip', c.overlay0, 14, 2.5);
            const m = await ui.hstack(r, 'match', { gap: 6, sizingH: 'fill' });
            for (const [k, v] of matches) await ui.chip(m, v, { key: k });
            await decision(r, tile);
            if (o.readOnly) await ui.button(r, 'Override', false, { small: true, ghost: true, color: c.accent });
            else { await ui.iconButton(r, 'edit', o.editing ? c.accent : c.overlay0); await ui.iconButton(r, 'trash', c.overlay0); }
            return r;
        };
        const mine = await ui.section(scroll, 'Your rules', { note: 'user-rules.toml' });
        await rule(mine, [['class is', 'MySpecialApp']], true);
        await rule(mine, [['executable', 'mytool.exe']], false, { editing: true, last: false });
        const form = await ui.frame(mine, 'rule editor', { layout: 'vertical', padding: [4, 16, 14, 40], gap: 12, fill: c.selected, sizingH: 'fill', sizingV: 'hug', strokeBottomWeight: 1, stroke: c.line });
        const fields = await ui.hstack(form, 'fields', { gap: 12, sizingH: 'fill' });
        for (const [label, value, hint] of [['Class is', '', 'Any'], ['Title contains', '', 'Any'], ['Executable path ends with', 'mytool.exe', '']]) {
            const f = await ui.vstack(fields, `field/${label}`, { gap: 5, sizingH: 'fill' });
            await ui.text(f, label, 11, c.subtext0);
            await ui.textField(f, value, 10, { placeholder: hint, fill: true, focused: !!value });
        }
        const actions = await ui.hstack(form, 'actions', { gap: 8, sizingH: 'fill' });
        await ui.text(actions, 'Matching windows', 12, c.subtext0);
        await ui.segmented(actions, ['Tile', "Don't tile"], "Don't tile", { changed: true });
        await ui.spacer(actions);
        await ui.button(actions, 'Done', false, { small: true, outline: true });
        await rule(mine, [['title contains', 'My Private Tool']], false, { last: true });
        const community = await ui.section(scroll, 'Community rules', { icon: 'lock', note: 'rules.toml, read-only. Downloaded from jmelosegui/mosaico-rules.', dim: true });
        await rule(community, [['class is', 'ApplicationFrameWindow'], ['title is', 'empty']], false, { readOnly: true });
        await rule(community, [['title contains', 'pinentry']], false, { readOnly: true, last: true });
        return { unsaved: 1 };
    },
};

module.exports = { PAGES };
