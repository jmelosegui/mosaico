// Prints WebVTT captions: each line from its start until its audio ends.
import { execFileSync } from "child_process";
import fs from "fs";

const lines = JSON.parse(fs.readFileSync("lines.json", "utf8"));
const stamp = (s) => {
  const m = Math.floor(s / 60), r = s - m * 60;
  return `${String(m).padStart(2, "0")}:${r.toFixed(3).padStart(6, "0")}`;
};
let vtt = "WEBVTT\n";
lines.forEach((line, i) => {
  const file = `out/vo/${String(i + 1).padStart(2, "0")}.mp3`;
  const length = Number(execFileSync("ffprobe", ["-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", file]).toString());
  vtt += `\n${stamp(line.start)} --> ${stamp(line.start + length)}\n${line.text}\n`;
});
process.stdout.write(vtt);
