#!/bin/zsh
# Rebuild every bench_shallow artifact: loon -> vcad (with/without panels) -> posed docs -> stills -> tables -> video/gif.
cd ${0:A:h}
R=${VCAD_RENDER:-vcad-render}
L2V=${LOON2VCAD:-loon2vcad}
python3 gen_bench_shallow.py > bench_shallow.loon && $L2V bench_shallow.loon > bench_shallow_raw.vcad || exit 1
python3 gen_bench_shallow.py --no-panels > bench_shallow_np.loon && $L2V bench_shallow_np.loon > bench_shallow_np_raw.vcad || exit 1
for p in TUCK MILL BENCH HANDOFF RACK WIND; do python3 frames.py pose $p bench_shallow_np_raw.vcad pose_$p.vcad; done
python3 frames.py pose TUCK bench_shallow_raw.vcad bench_shallow_enclosed.vcad
B1='mill,mill_stand,bay1_*'; B2='bench,bench_legs,reflow,bay2_*'; B3='winder,stock,finished,bay3_*'
python3 subset.py pose_MILL.vcad v_mill.vcad "human,table,$B2,$B3"
python3 subset.py pose_BENCH.vcad v_work.vcad "human,table,$B1,$B3"
python3 subset.py pose_RACK.vcad v_rack.vcad "human,table,$B1,$B2,$B3,curtain_*"
python3 subset.py pose_TUCK.vcad v_bay.vcad --keep "deck,rail_f,chain,busbar,bench,bench_legs,reflow,bay2_*" --hide-frame
./render_labeled.sh pose_TUCK.vcad bench_shallow_iso.png --azimuth 215 --elevation 25 --size 1800 &
./render_labeled.sh pose_TUCK.vcad bench_shallow_top.png --view top --dims --size 1600 &
./render_labeled.sh pose_TUCK.vcad bench_shallow_front.png --view front --size 1600 &
$R v_mill.vcad --azimuth 20 --elevation 22 --section x=300 --size 1400 -o bench_shallow_mill.png 2>/dev/null &   # section cut through the mill: labels do not survive the cut
HIDE=tool_strip,light,cam,curtain_r,curtain_l ./render_labeled.sh v_work.vcad bench_shallow_work.png --azimuth 292 --elevation 38 --size 1400 &
HIDE=light,cam ./render_labeled.sh v_rack.vcad bench_shallow_rack.png --azimuth 250 --elevation 22 --size 1400 &
$R bench_shallow_enclosed.vcad --azimuth 215 --elevation 25 --size 1600 -o bench_shallow_enclosed.png 2>/dev/null &
$R v_bay.vcad --azimuth 90 --elevation 22 --labels --size 1400 -o bench_shallow_bay.png 2>/dev/null &   # few parts: direct labels
wait
python3 frames.py table > joints.txt; python3 reach.py > reach.txt; python3 loads.py > loads.txt
[[ -n $STILLS_ONLY ]] && exit 0
python3 subset.py bench_shallow_np_raw.vcad video_base.vcad human,table
python3 frames.py
ffmpeg -y -loglevel error -framerate 24 -i vfr2/%04d.jpg -vf crop=1280:720:160:40 -c:v libx264 -pix_fmt yuv420p -crf 20 bench_shallow_working.mp4
ffmpeg -y -loglevel error -i bench_shallow_working.mp4 -vf "setpts=PTS*6/14.6,fps=12,scale=640:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=96[p];[b][p]paletteuse" bench_shallow_working.gif
