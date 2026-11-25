// Prints WebVTT captions: each line from its start until its audio ends.
// INTRO is the length of the logo card build.sh puts before the video.
import { execFileSync } from "child_process";
import fs from "fs";

const lines = JSON.parse(fs.readFileSync("lines.json", "utf8"));
const intro = Number(process.env.INTRO ?? 0);
const stamp = (s) => {
  const m = Math.floor(s / 60), r = s - m * 60;
  return `${String(m).padStart(2, "0")}:${r.toFixed(3).padStart(6, "0")}`;
};
let vtt = "WEBVTT\n";
lines.forEach((line, i) => {
  const file = `out/vo/${String(i + 1).padStart(2, "0")}.mp3`;
  const length = Number(execFileSync("ffprobe", ["-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", file]).toString());
  vtt += `\n${stamp(intro + line.start)} --> ${stamp(intro + line.start + length)}\n${line.text}\n`;
});
process.stdout.write(vtt);
