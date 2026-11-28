#!/usr/bin/env bash
# Rebuilds the demo video, its captions and the poster into ../assets.
# Needs node, ffmpeg and Chrome (set CHROME if it is not in the default place).
# The Chrome fonts must include Segoe UI, Cascadia Code and CaskaydiaCove Nerd Font.
set -euo pipefail
cd "$(dirname "$0")"

npm ci --ignore-scripts
node tts.mjs
node music.mjs
node capture.mjs 75
ffmpeg -y -v error -framerate 30 -i out/frames/f%05d.jpg -c:v libx264 -pix_fmt yuv420p -crf 18 -preset slow out/visuals.mp4

# Narration lines start at their "start" times; the music ducks under them.
starts=($(node -e 'for (const l of require("./lines.json")) console.log(l.start)'))
inputs=() filters="" mix=""
for i in "${!starts[@]}"; do
  ms=$(node -e "console.log(Math.round(${starts[$i]} * 1000))")
  inputs+=(-i "out/vo/$(printf %02d $((i + 1))).mp3")
  filters+="[$((i + 2)):a]aresample=48000,adelay=${ms}|${ms},apad[v$i];"
  mix+="[v$i]"
done
ffmpeg -y -v error -i out/visuals.mp4 -i out/music.wav "${inputs[@]}" -filter_complex \
  "${filters}${mix}amix=inputs=${#starts[@]}:normalize=0,atrim=0:75,volume=1.6,asplit=2[vo][sc];[1:a]volume=0.55[mu];[mu][sc]sidechaincompress=threshold=0.03:ratio=6:attack=20:release=400[duck];[duck][vo]amix=inputs=2:normalize=0,loudnorm=I=-16:TP=-1.5:LRA=11[a]" \
  -map 0:v -map "[a]" -c:v copy -c:a aac -b:a 160k -ar 48000 -shortest out/body.mp4

# The end card opens the video too, so players that show the first frame
# before it plays (GitHub's among them) show the logo. It fades into the
# first scene, and the sound starts INTRO seconds later to match.
INTRO=0.6
ffmpeg -y -v error -loop 1 -framerate 30 -t 1 -i out/frames/f02249.jpg -i out/body.mp4 -filter_complex \
  "[0:v]format=yuv420p,setsar=1,fps=30,settb=1/30[i];[1:v]setsar=1,fps=30,settb=1/30[b];[i][b]xfade=transition=fade:duration=0.4:offset=$INTRO[v];[1:a]adelay=$(node -e "console.log($INTRO * 1000)"):all=1[a]" \
  -map "[v]" -map "[a]" -c:v libx264 -pix_fmt yuv420p -crf 18 -preset slow -c:a aac -b:a 160k -ar 48000 -movflags +faststart ../assets/mosaico-demo.mp4

ffmpeg -y -v error -i ../assets/mosaico-demo.mp4 -frames:v 1 -q:v 3 ../assets/first-frame.jpg
INTRO=$INTRO node captions.mjs > ../assets/mosaico-demo.en.vtt
