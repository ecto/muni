#!/bin/zsh
# Rebuild every cell_v2 artifact: loon -> vcad (with / without panels) -> posed docs -> stills.
cd ${0:A:h}
R=${VCAD_RENDER:-vcad-render}
L2V=${LOON2VCAD:-loon2vcad}
python3 gen_cell_v2.py > cell_v2.loon && $L2V cell_v2.loon > cell_v2_raw.vcad || exit 1
python3 gen_cell_v2.py --no-panels > cell_v2_np.loon && $L2V cell_v2_np.loon > cell_v2_np_raw.vcad || exit 1
for p in TUCK MILL BENCH HANDOFF OVEN; do python3 make_frames_v2.py pose $p cell_v2_np_raw.vcad pose_$p.vcad; done
python3 make_frames_v2.py pose TUCK cell_v2_raw.vcad cell_v2_enclosed.vcad
./render_labeled_v2.sh pose_TUCK.vcad cell_v2_iso.png --azimuth 220 --elevation 28 --size 1600 &
./render_labeled_v2.sh pose_TUCK.vcad cell_v2_top.png --view top --dims --size 1600 &
$R cell_v2_enclosed.vcad --azimuth 220 --elevation 28 --size 1600 -o cell_v2_iso_enclosed.png 2>/dev/null &
$R pose_MILL.vcad --azimuth 205 --elevation 22 --size 1400 -o cell_v2_mill.png 2>/dev/null &
$R pose_MILL.vcad --azimuth 200 --elevation 20 --focus mill --size 1000 -o cell_v2_mill_close.png 2>/dev/null &
$R pose_HANDOFF.vcad --azimuth 250 --elevation 30 --focus plinth --size 1400 -o cell_v2_bench.png 2>/dev/null &
$R pose_BENCH.vcad --azimuth 250 --elevation 30 --focus plinth --size 1400 -o cell_v2_bench_work.png 2>/dev/null &
$R pose_OVEN.vcad --azimuth 330 --elevation 22 --focus reflow --size 1400 -o cell_v2_oven.png 2>/dev/null &
python3 -c "
import json;d=json.load(open('pose_TUCK.vcad'));keep=('frame','floor','deck','psu','canhub','controller')
d['instances']=[i for i in d['instances'] if i['id'] in keep];d['joints']=[j for j in d['joints'] if j['childInstanceId'] in keep]
json.dump(d,open('bay.vcad','w'))"
BAY=1 ./render_labeled_v2.sh bay.vcad cell_v2_bay.png --azimuth 220 --elevation 35 --section z=170 --size 1200 &
wait
# analysis tables + working video (frames rendered in parallel by make_frames_v2.py)
python3 make_frames_v2.py table > joints_v2.txt
python3 reach_v2.py > reach_v2.txt
python3 loads_v2.py > loads_v2.txt
python3 make_frames_v2.py
ffmpeg -y -loglevel error -framerate 24 -i frames_v2/%04d.png -c:v libx264 -pix_fmt yuv420p -crf 20 cell_v2_working.mp4
ffmpeg -y -loglevel error -i cell_v2_working.mp4 -vf "setpts=PTS*6/14,fps=12,scale=640:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=96[p];[b][p]paletteuse" cell_v2_working.gif
