// Widgets for the settings sketches: ArtCraft shapes and spacing, theme colors.
const { ICONS, WIDE_ICONS } = require('./theme.js');

function makeUi(fg, c) {
    // FigCraft draws per-side strokes from strokeSides [top, right, bottom, left].
    const SIDES = ['strokeTopWeight', 'strokeRightWeight', 'strokeBottomWeight', 'strokeLeftWeight'];
    const node = async (type, parent, props = {}, extra = {}) => {
        if (SIDES.some((k) => k in props)) {
            props = { ...props, strokeWeight: 1, strokeSides: SIDES.map((k) => props[k] || 0) };
            for (const k of SIDES) delete props[k];
        }
        return (await fg.call('create_node', { type, parent, props, select: false, ...extra })).id;
    };
    const frame = (parent, name, props = {}, extra = {}) =>
        node('frame', parent, { fill: 'none', ...props }, { name, ...extra });
    const text = (parent, s, size = 13, color = c.text, style = 'Regular', props = {}) =>
        node('text', parent, { fontFamily: 'Inter', fontStyle: style, fontSize: size, fill: color, ...props }, { text: s });
    const icon = async (parent, key, color, size = 16, weight = 1.75) => {
        const box = await frame(parent, `icon/${key}`, { layout: 'horizontal', mainAlign: 'center', crossAlign: 'center' }, { width: size, height: size });
        const dim = WIDE_ICONS.has(key) ? { width: size * 0.8 } : { height: size * 0.8 };
        await node('vector', box, { fill: 'none', stroke: color, strokeWeight: weight, strokeCap: 'round', strokeJoin: 'round' }, { path: ICONS[key], ...dim, name: key });
        return box;
    };
    const spacer = (parent) => frame(parent, 'spacer', { sizingH: 'fill', sizingV: 'fill' }, { width: 1, height: 1 });
    const hstack = (parent, name, props = {}) => frame(parent, name, { layout: 'horizontal', crossAlign: 'center', sizingH: 'hug', sizingV: 'hug', ...props });
    const vstack = (parent, name, props = {}) => frame(parent, name, { layout: 'vertical', sizingH: 'hug', sizingV: 'hug', ...props });
    const divider = (parent) => node('rectangle', parent, { fill: c.line, sizingH: 'fill' }, { width: 10, height: 1, name: 'divider' });
    const image = async (parent, path, size, name) => {
        await fg.call('run_command', { command: 'image.place', params: { path, parent, width: size, height: size, name } });
    };
    // Position of a layer relative to an ancestor, once auto layout has settled.
    const boundsIn = async (ancestor, id) => {
        const a = (await fg.call('get_node', { id: ancestor, depth: 0 })).absoluteBounds;
        const b = (await fg.call('get_node', { id, depth: 0 })).absoluteBounds;
        return { x: b.x - a.x, y: b.y - a.y, width: b.width, height: b.height };
    };

    // Controls
    const swatch = async (parent, hex, size = 14, props = {}) => {
        if (hex === 'none') {
            const f = await frame(parent, 'swatch/none', { fill: c.control, stroke: c.overlay0, strokeWeight: 1, cornerRadius: 4, ...props }, { width: size, height: size });
            await node('vector', f, { absolute: true, fill: 'none', stroke: c.red, strokeWeight: 1.5, strokeCap: 'round' }, { path: `M${size - 3} 3L3 ${size - 3}`, x: 3, y: 3, height: size - 6, name: 'slash' });
            return f;
        }
        return node('rectangle', parent, { fill: hex, cornerRadius: 4, stroke: c.line, strokeWeight: 1, ...props }, { width: size, height: size, name: `swatch/${hex}` });
    };
    const dropdown = async (parent, value, width = 168, o = {}) => {
        const f = await frame(parent, `dropdown/${value}`, { layout: 'horizontal', fill: c.control, cornerRadius: 5, padding: [0, 8, 0, 10], gap: 8, crossAlign: 'center', opacity: o.disabled ? 0.45 : 1, ...(o.open || o.changed ? { stroke: c.accent, strokeWeight: 1 } : {}) }, { width, height: 26 });
        if (o.swatch) await swatch(f, o.swatch, 14);
        if (o.icon) await icon(f, o.icon, c.subtext0, 14);
        await text(f, value, o.size || 13, o.dim ? c.overlay1 : c.text, 'Regular', { sizingH: 'fill' });
        await icon(f, o.open ? 'up' : 'chevron', c.subtext0, 14);
        return f;
    };
    const toggle = async (parent, on) => {
        const f = await frame(parent, `toggle/${on ? 'on' : 'off'}`, { layout: 'horizontal', fill: on ? c.accent : c.control, cornerRadius: 10, ...(on ? {} : { stroke: c.line, strokeWeight: 1 }) }, { width: 34, height: 20 });
        await node('ellipse', f, { absolute: true, fill: on ? c.base : c.overlay1 }, { x: on ? 17 : 3, y: 3, width: 14, height: 14, name: 'knob' });
        return f;
    };
    const checkbox = async (parent, on, label) => {
        const h = await hstack(parent, `checkbox/${label || ''}`, { gap: 8 });
        const box = await frame(h, 'box', { layout: 'horizontal', mainAlign: 'center', crossAlign: 'center', cornerRadius: 4, ...(on ? { fill: c.accent } : { fill: c.control, stroke: c.overlay0, strokeWeight: 1 }) }, { width: 16, height: 16 });
        if (on) await icon(box, 'check', c.base, 12, 2.25);
        if (label) await text(h, label, 13);
        return h;
    };
    const segmented = async (parent, options, selected, o = {}) => {
        const f = await frame(parent, 'segmented', { layout: 'horizontal', fill: c.control, cornerRadius: 6, padding: 2, gap: 2, sizingH: 'hug', sizingV: 'hug', ...(o.changed ? { stroke: c.accent, strokeWeight: 1 } : {}) });
        for (const opt of options) {
            const on = opt === selected;
            const item = await frame(f, `segment/${opt}`, { layout: 'horizontal', fill: on ? c.surface1 : 'none', cornerRadius: 4, padding: [4, 10, 4, 10], sizingH: 'hug', sizingV: 'hug', crossAlign: 'center', gap: 6 });
            await text(item, opt, 12, on ? c.text : c.subtext0, on ? 'SemiBold' : 'Regular');
        }
        return f;
    };
    const spin = async (parent, value, unit, o = {}) => {
        const f = await frame(parent, `spin/${value}`, { layout: 'horizontal', fill: c.control, cornerRadius: 5, padding: [0, 8, 0, 10], gap: 6, crossAlign: 'center', ...(o.changed ? { stroke: c.accent, strokeWeight: 1 } : {}) }, { width: o.width || 96, height: 26 });
        await text(f, value, 13, c.text, 'Regular', { sizingH: 'fill' });
        if (unit) await text(f, unit, 12, c.overlay0);
        return f;
    };
    const slider = async (parent, fraction, label, width = 200) => {
        // Left padding leaves room for the thumb when the value is 0.
        const f = await hstack(parent, 'slider', { gap: 12, padding: [0, 0, 0, 7], clip: false });
        const rail = await frame(f, 'rail', { layout: 'horizontal', fill: c.control, cornerRadius: 2, clip: false }, { width, height: 4 });
        await node('rectangle', rail, { absolute: true, fill: c.accent, cornerRadius: 2 }, { x: 0, y: 0, width: Math.max(1, Math.round(width * fraction)), height: 4, name: 'filled' });
        await node('ellipse', rail, { absolute: true, fill: c.text, stroke: c.accent, strokeWeight: 2 }, { x: Math.round(width * fraction) - 7, y: -5, width: 14, height: 14, name: 'thumb' });
        const v = await frame(f, 'value', { layout: 'horizontal', fill: c.control, cornerRadius: 5, padding: [3, 8, 3, 8], sizingH: 'hug', sizingV: 'hug' });
        await text(v, label, 12);
        return f;
    };
    const textField = async (parent, value, width = 200, o = {}) => {
        const f = await frame(parent, `field/${value || o.placeholder}`, { layout: 'horizontal', fill: c.control, cornerRadius: 5, padding: [0, 10, 0, 10], gap: 8, crossAlign: 'center', sizingH: o.fill ? 'fill' : 'fixed', ...(o.focused ? { stroke: c.accent, strokeWeight: 1 } : {}) }, { width, height: 26 });
        if (o.icon) await icon(f, o.icon, c.overlay0, 14);
        await text(f, value || o.placeholder, 13, value ? c.text : c.overlay0, 'Regular', { sizingH: 'fill' });
        if (o.hint) await text(f, o.hint, 11, c.overlay0);
        return f;
    };
    const button = async (parent, label, primary, o = {}) => {
        const f = await frame(parent, `button/${label}`, { layout: 'horizontal', fill: primary ? c.accent : o.ghost ? 'none' : c.surface0, cornerRadius: 5, padding: o.small ? [3, 10, 3, 10] : [6, 16, 6, 16], gap: 6, crossAlign: 'center', sizingH: 'hug', sizingV: 'hug', opacity: o.disabled ? 0.45 : 1, ...(o.outline ? { stroke: c.line, strokeWeight: 1 } : {}) });
        const color = primary ? c.base : o.color || c.text;
        if (o.icon) await icon(f, o.icon, color, 14);
        await text(f, label, o.small ? 12 : 13, color, 'SemiBold');
        return f;
    };
    const iconButton = async (parent, key, color = c.subtext0, o = {}) => {
        const f = await frame(parent, `icon button/${key}`, { layout: 'horizontal', cornerRadius: 5, fill: o.fill || 'none', mainAlign: 'center', crossAlign: 'center' }, { width: 26, height: 26 });
        await icon(f, key, color, 14);
        return f;
    };
    const chip = async (parent, label, o = {}) => {
        const f = await hstack(parent, `chip/${label}`, { fill: o.fill || c.control, cornerRadius: 10, padding: [3, 9, 3, 9], gap: 5, ...(o.stroke ? { stroke: o.stroke, strokeWeight: 1 } : {}) });
        if (o.key) await text(f, o.key, 11, c.overlay1);
        await text(f, label, 11, o.color || c.text, o.strong ? 'SemiBold' : 'Regular');
        return f;
    };
    const keycap = async (parent, label) => {
        const f = await frame(parent, `key/${label}`, { layout: 'horizontal', fill: c.control, stroke: c.line, strokeWeight: 1, cornerRadius: 4, padding: [2, 7, 2, 7], mainAlign: 'center', sizingH: 'hug', sizingV: 'hug' });
        await text(f, label, 11, c.text, 'SemiBold');
        return f;
    };
    const keys = async (parent, list, o = {}) => {
        const f = await hstack(parent, 'keys', { gap: 4, ...(o.capturing ? { padding: [3, 6, 3, 6], cornerRadius: 6, stroke: c.accent, strokeWeight: 1, fill: c.control } : {}) });
        for (const k of list) await keycap(f, k);
        return f;
    };
    const tabs = async (parent, items, selected) => {
        const f = await frame(parent, 'tabs', { layout: 'horizontal', gap: 20, padding: [14, 0, 0, 0], sizingH: 'fill', sizingV: 'hug', strokeBottomWeight: 1, stroke: c.line });
        for (const t of items) {
            const on = t === selected;
            const tab = await frame(f, `tab/${t}`, { layout: 'vertical', gap: 7, sizingH: 'hug', sizingV: 'hug', padding: [0, 0, 0, 0] });
            await text(tab, t, 13, on ? c.text : c.subtext0, on ? 'SemiBold' : 'Regular');
            await node('rectangle', tab, { fill: on ? c.accent : 'none', cornerRadius: 1, sizingH: 'fill' }, { width: 10, height: 2, name: 'underline' });
        }
        return f;
    };
    const popover = (parent, x, y, width, name) =>
        frame(parent, name, { absolute: true, layout: 'vertical', fill: c.card, stroke: c.line, strokeWeight: 1, cornerRadius: 8, padding: 12, gap: 10, sizingV: 'hug', effects: [{ type: 'dropShadow', color: c.dark ? '#00000080' : '#4C4F6933', offset: [0, 8], radius: 24 }] }, { x, y, width });

    // Page structure
    const section = async (parent, title, o = {}) => {
        const s = await frame(parent, `section/${title}`, { layout: 'vertical', gap: 8, sizingH: 'fill', sizingV: 'hug' });
        const head = await hstack(s, 'heading', { gap: 8, sizingH: 'fill' });
        if (o.icon) await icon(head, o.icon, c.subtext0, 13);
        await text(head, title, 13, c.subtext0, 'SemiBold');
        if (o.note) await text(head, o.note, 11, c.overlay0);
        if (o.right) { await spacer(head); await o.right(head); }
        if (o.help) await text(s, o.help, 11, c.subtext0, 'Regular', { sizingH: 'fill', textAutoResize: 'height' });
        return frame(s, 'card', { layout: 'vertical', fill: c.card, stroke: c.line, strokeWeight: 1, cornerRadius: 8, sizingH: 'fill', sizingV: 'hug', clip: true, opacity: o.dim ? 0.8 : 1 });
    };
    const row = async (card, label, help, control, o = {}) => {
        const pad = o.compact ? [8, 16, 8, 16] : [12, 16, 12, 16];
        const r = await frame(card, `row/${label}`, { layout: 'horizontal', padding: pad, gap: 16, crossAlign: 'center', sizingH: 'fill', sizingV: 'hug', ...(o.last ? {} : { strokeBottomWeight: 1, stroke: c.line }), ...(o.fill ? { fill: o.fill } : {}) });
        const left = await frame(r, 'label', { layout: 'vertical', gap: 3, sizingH: 'fill', sizingV: 'hug' });
        const head = await hstack(left, 'title', { gap: 6 });
        await text(head, label, 13, c.text, o.compact ? 'Regular' : 'SemiBold');
        if (o.sub) await text(head, o.sub, 11, c.overlay0);
        if (o.changed) await node('ellipse', head, { fill: c.accent }, { width: 6, height: 6, name: 'changed' });
        if (help) await text(left, help + (o.def ? `  Default: ${o.def}.` : ''), 11, c.subtext0, 'Regular', { sizingH: 'fill', textAutoResize: 'height' });
        if (o.changed) await iconButton(r, 'reset', c.accent, { fill: c.control });
        if (control) await control(r);
        return r;
    };

    return { node, frame, text, icon, spacer, hstack, vstack, divider, image, boundsIn, swatch, dropdown, toggle, checkbox, segmented, spin, slider, textField, button, iconButton, chip, keycap, keys, tabs, popover, section, row };
}

module.exports = { makeUi };
