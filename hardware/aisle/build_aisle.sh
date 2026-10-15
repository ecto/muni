#!/bin/zsh
# Rebuild the aisle: generate -> loon2vcad -> posed docs -> stills. Times printed.
cd ${0:A:h}
R=${VCAD_RENDER:-vcad-render}
L2V=${LOON2VCAD:-loon2vcad}
python3 gen_aisle.py && time $L2V aisle.loon > aisle_raw.vcad || exit 1
python3 make_frames_aisle.py pose T1_HOME+T2_HOME aisle_park.vcad
python3 make_frames_aisle.py pose T1_TOPSHELF_MAX+T2_HOME aisle_rack.vcad
python3 make_frames_aisle.py pose T1_SAW+T2_BENCH aisle_bench.vcad
time $R aisle_park.vcad --azimuth 215 --elevation 25 --size 1800 -o t_time.png 2>/dev/null
./render_labeled_aisle.sh aisle_park.vcad aisle_iso.png --azimuth 215 --elevation 25 --size 2400x1500 &
BAYS=1 ./render_labeled_aisle.sh aisle_park.vcad aisle_top.png --view top --dims --size 2600x1400 &
./render_labeled_aisle.sh aisle_park.vcad aisle_front.png --azimuth 180 --elevation 10 --size 1600 &
python3 subset_doc.py aisle_rack.vcad sub_rack.vcad 2700 4300 1000 2750 human,light_rail,cam_1,cam_2,cam_3,cam_4,fence_back
$R sub_rack.vcad --azimuth 290 --elevation 15 --size 1600 -o aisle_rack.png 2>/dev/null &
python3 subset_doc.py aisle_bench.vcad sub_bench.vcad 900 2600 -50 1700 human,light_rail,cam_1,cam_2,cam_3,cam_4,fence_front
$R sub_bench.vcad --azimuth 75 --elevation 32 --size 1600 -o aisle_bench_bay.png 2>/dev/null &
wait
for f in aisle_*.png; do magick $f -background white -flatten $f; done
# working video (frames rendered in parallel by make_frames_aisle.py)
python3 make_frames_aisle.py frames
ffmpeg -y -loglevel error -framerate 24 -i frames/%04d.png -c:v libx264 -pix_fmt yuv420p -crf 20 aisle_working.mp4
ffmpeg -y -loglevel error -i aisle_working.mp4 -vf "setpts=PTS*6/16,fps=12,scale=640:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=96[p];[b][p]paletteuse" aisle_working.gif
