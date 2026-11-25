// Synthesizes the music bed: a tense drone, a hit on "Enough.", then a
// brighter chord progression with a soft arpeggio. Writes out/music.wav.
import fs from "fs";
const SR = 48000, DUR = 75, N = Math.floor(SR * DUR);
const L = new Float32Array(N), R = new Float32Array(N);
const add = (i, v, pan = 0) => { if (i >= 0 && i < N) { L[i] += v * (1 - pan) ; R[i] += v * (1 + pan); } };
const env = (t, a, len, att, rel) => { const x = t - a; if (x < 0 || x > len) return 0; return Math.min(1, x / att, (len - x) / rel); };
function pad(start, len, freqs, amp, att = 1.2, rel = 1.2) {
  for (let i = Math.floor(start * SR); i < Math.floor((start + len) * SR); i++) {
    const t = i / SR, e = env(t, start, len, att, rel); let v = 0;
    freqs.forEach((f, k) => { v += Math.sin(2 * Math.PI * f * t) + .35 * Math.sin(2 * Math.PI * f * 2.003 * t) + .5 * Math.sin(2 * Math.PI * f * 1.004 * t + k); });
    add(i, v * amp * e / freqs.length);
  }
}
function pluck(start, f, amp, pan) {
  for (let i = 0; i < SR * 1.2; i++) { const t = i / SR; add(Math.floor(start * SR) + i, amp * Math.exp(-t * 5) * (Math.sin(2 * Math.PI * f * t) + .3 * Math.sin(4 * Math.PI * f * t)), pan); }
}
function boom(start, amp) {
  for (let i = 0; i < SR * 2.5; i++) { const t = i / SR; const f = 60 * Math.exp(-t * 1.5) + 38; add(Math.floor(start * SR) + i, amp * Math.exp(-t * 2.2) * Math.sin(2 * Math.PI * f * t)); }
}
// chaos: uneasy drone with tremolo
for (let i = 0; i < Math.floor(13.85 * SR); i++) {
  const t = i / SR, e = Math.min(1, t / 2, (13.85 - t) / .05), trem = .75 + .25 * Math.sin(2 * Math.PI * (2 + t * .5) * t);
  let v = Math.sin(2 * Math.PI * 110 * t) + .6 * Math.sin(2 * Math.PI * 116.5 * t) + .4 * Math.sin(2 * Math.PI * 164.8 * t);
  add(i, v * .12 * e * trem);
}
boom(13.85, .7);
// bright section: C G Am F
const C = [130.81, 196, 261.63, 329.63], G = [98, 196, 246.94, 293.66], Am = [110, 220, 261.63, 329.63], F = [87.31, 174.61, 261.63, 349.23];
const prog = [C, G, Am, F, C, G, F, C, Am, F, G, C];
const start = 15.1, bar = 5.3;
prog.forEach((ch, k) => {
  const s = start + k * bar, len = k === prog.length - 1 ? DUR - s : bar + .6;
  pad(s, len, ch, .16, k === 0 ? 2 : .5, k === prog.length - 1 ? 3 : .6);
  if (k >= 1 && s < 68) {
    const arp = [ch[2], ch[3], ch[1] * 2, ch[3]];
    for (let j = 0; j < 16; j++) pluck(s + j * bar / 16, arp[j % 4] * 2, .05, j % 2 ? .3 : -.3);
  }
});
boom(15.2, .35); boom(69.3, .45);
// normalize to -3 dBFS and write 16-bit wav
let peak = 0; for (let i = 0; i < N; i++) peak = Math.max(peak, Math.abs(L[i]), Math.abs(R[i]));
const g = .7 / peak, buf = Buffer.alloc(44 + N * 4);
buf.write("RIFF", 0); buf.writeUInt32LE(36 + N * 4, 4); buf.write("WAVEfmt ", 8); buf.writeUInt32LE(16, 16); buf.writeUInt16LE(1, 20); buf.writeUInt16LE(2, 22);
buf.writeUInt32LE(SR, 24); buf.writeUInt32LE(SR * 4, 28); buf.writeUInt16LE(4, 32); buf.writeUInt16LE(16, 34); buf.write("data", 36); buf.writeUInt32LE(N * 4, 40);
for (let i = 0; i < N; i++) { buf.writeInt16LE(Math.round(L[i] * g * 32767), 44 + i * 4); buf.writeInt16LE(Math.round(R[i] * g * 32767), 46 + i * 4); }
fs.mkdirSync("out", { recursive: true });
fs.writeFileSync("out/music.wav", buf);
