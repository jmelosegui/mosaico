// Renders scene.html frame by frame into out/frames, 30 frames a second.
// FROM and TO limit the render to a range of frame numbers.
import puppeteer from "puppeteer-core";
import fs from "fs";
import path from "path";
import { pathToFileURL } from "url";

const seconds = Number(process.argv[2] ?? 75);
const from = Number(process.env.FROM ?? 0), to = Number(process.env.TO ?? Infinity);
const browser = await puppeteer.launch({
  executablePath: process.env.CHROME ?? "C:/Program Files/Google/Chrome/Application/chrome.exe",
  args: ["--force-device-scale-factor=1", "--allow-file-access-from-files"],
});
const page = await browser.newPage();
await page.setViewport({ width: 1920, height: 1080 });
await page.goto(pathToFileURL(path.resolve("scene.html")).href + "?t=0");
await page.evaluate(() => document.fonts.ready);
fs.mkdirSync("out/frames", { recursive: true });
for (let n = from; n < Math.min(to, Math.round(seconds * 30)); n++) {
  await page.evaluate((t) => render(t), n / 30);
  await page.screenshot({ path: `out/frames/f${String(n).padStart(5, "0")}.jpg`, type: "jpeg", quality: 92 });
}
await browser.close();
