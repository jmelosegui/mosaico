// Saves the open FigCraft document as settings.figcraft and renders every
// "Settings / ..." and "Tray / ..." window to png/<name>.png.
//   node design/settings/export.js            render all and save
//   node design/settings/export.js Borders    render matching windows only, no save
const fs = require('fs');
const path = require('path');
const { connect } = require(path.join(__dirname, 'fgsession.js'));

const slug = (s) => s.toLowerCase().normalize('NFD').replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');

async function main() {
    const filters = process.argv.slice(2);
    const fg = connect();
    const doc = await fg.call('inspect_document', {});
    const page = await fg.call('get_node', { id: doc.currentPage, depth: 1 });
    const windows = (page.children || [])
        .filter((n) => /^(Settings|Tray) \/ /.test(n.name))
        .filter((n) => !filters.length || filters.some((f) => n.name.includes(f)));
    fs.mkdirSync(path.join(__dirname, 'png'), { recursive: true });
    for (const w of windows) {
        const out = path.join(__dirname, 'png', `${slug(w.name.replace(/^Settings \/ /, ''))}.png`);
        const shot = await fg.call('screenshot', { ids: [w.id], scale: 2 });
        fs.writeFileSync(out, Buffer.from(shot.image, 'base64'));
        console.log(path.relative(process.cwd(), out));
    }
    if (!filters.length) {
        await fg.call('select', { ids: [] });
        await fg.call('save_file', { path: path.join(__dirname, 'settings.figcraft') });
        console.log(path.relative(process.cwd(), path.join(__dirname, 'settings.figcraft')));
    }
    fg.close();
}

main().catch((e) => { console.error(e.message); process.exit(1); });
