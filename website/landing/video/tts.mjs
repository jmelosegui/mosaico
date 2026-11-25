// Speaks each line of lines.json into out/vo/NN.mp3 with Edge's neural voice.
// "say" overrides "text" where the spoken form needs a respelling.
import { MsEdgeTTS, OUTPUT_FORMAT } from "msedge-tts";
import fs from "fs";

const lines = JSON.parse(fs.readFileSync("lines.json", "utf8"));
fs.mkdirSync("out/vo", { recursive: true });
for (const [i, line] of lines.entries()) {
  const tts = new MsEdgeTTS();
  await tts.setMetadata("en-US-AndrewNeural", OUTPUT_FORMAT.AUDIO_24KHZ_96KBITRATE_MONO_MP3);
  const { audioStream } = tts.toStream(line.say ?? line.text, { rate: "-5%", pitch: "-4Hz" });
  const file = fs.createWriteStream(`out/vo/${String(i + 1).padStart(2, "0")}.mp3`);
  await new Promise((resolve, reject) => {
    audioStream.pipe(file);
    file.on("finish", resolve);
    audioStream.on("error", reject);
  });
  tts.close();
}
