// Rebuilds every sketch in the open FigCraft document and exports them.
//   node design/settings/all.js
// One row per page, Mocha on the left and Latte on the right; the tray to the right.
const path = require('path');
const { execFileSync } = require('child_process');

const ORDER = ['General', 'Layout', 'Workspaces', 'Borders', 'Mouse', 'Animations', 'Bar', 'Bar widgets', 'Bar mouse', 'Keybindings', 'Rules'];
const run = (script, args) => process.stdout.write(execFileSync(process.execPath, [path.join(__dirname, script), ...args.map(String)]));

for (const [i, page] of ORDER.entries()) {
    run('build.js', [page, 'mocha', 0, i * 780]);
    run('build.js', [page, 'latte', 1060, i * 780]);
}
run('tray.js', [2120, 0]);
run('export.js', []);
