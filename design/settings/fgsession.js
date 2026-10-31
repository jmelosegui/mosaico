// One MCP session attached to the running FigCraft app, with await-able calls.
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

// FIGCRAFT_CLI wins, then figcraft-cli on the PATH, then the folder the Windows installer uses.
function findCli() {
    if (process.env.FIGCRAFT_CLI) return process.env.FIGCRAFT_CLI;
    const name = process.platform === 'win32' ? 'figcraft-cli.exe' : 'figcraft-cli';
    const dirs = (process.env.PATH || '').split(path.delimiter).filter(Boolean);
    if (process.platform === 'win32' && process.env.ProgramFiles) dirs.push(path.join(process.env.ProgramFiles, 'FigCraft'));
    const found = dirs.map((d) => path.join(d, name)).find((p) => fs.existsSync(p));
    if (!found) throw new Error(`${name} not found. Add FigCraft to the PATH or set FIGCRAFT_CLI to its path.`);
    return found;
}

function connect(port = process.env.FG_PORT || '7879') {
    const child = spawn(findCli(), ['mcp', '--attach', port, '--timeout', '120']);
    const pending = new Map();
    let next = 1;
    let buffer = '';
    child.stdout.on('data', (data) => {
        buffer += data;
        let end;
        while ((end = buffer.indexOf('\n')) >= 0) {
            const line = buffer.slice(0, end);
            buffer = buffer.slice(end + 1);
            const m = JSON.parse(line);
            const p = pending.get(m.id);
            if (!p) continue;
            pending.delete(m.id);
            if (m.error) return p.reject(new Error(JSON.stringify(m.error)));
            const parts = m.result?.content || [];
            const text = parts.filter((x) => x.type === 'text').map((x) => x.text).join('\n');
            const image = parts.find((x) => x.type === 'image');
            if (m.result?.isError) return p.reject(new Error(`${p.name}: ${text}`));
            let value = text;
            try { value = JSON.parse(text); } catch {}
            p.resolve(image ? { value, image: image.data } : value);
        }
    });
    child.stderr.on('data', () => {});
    const raw = (method, params) => new Promise((resolve, reject) => {
        const id = next++;
        pending.set(id, { resolve, reject, name: params?.name });
        child.stdin.write(JSON.stringify({ jsonrpc: '2.0', id, method, params }) + '\n');
    });
    const ready = raw('initialize', { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'build', version: '1' } })
        .then(() => child.stdin.write(JSON.stringify({ jsonrpc: '2.0', method: 'notifications/initialized' }) + '\n'));
    return {
        call: async (name, args = {}) => { await ready; return raw('tools/call', { name, arguments: args }); },
        close: () => child.stdin.end(),
    };
}

module.exports = { connect };
