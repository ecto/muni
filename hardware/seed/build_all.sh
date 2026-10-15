#!/bin/zsh
# Rebuild the bay interface (bay_*.png) and the seed (seed_*.png + seed_growth.mp4/gif).
cd ${0:A:h}
export PATH=/opt/homebrew/bin:$PATH
L2V=${LOON2VCAD:-loon2vcad}
R=${VCAD_RENDER:-vcad-render}
python3 bay.py > bay.loon && $L2V bay.loon > bay.vcad || exit 1          # also writes bay_names.txt
python3 bay.py --one > bay_one.loon && $L2V bay_one.loon > bay_one_raw.vcad || exit 1
python3 bay_docs.py
time ./build_bay.sh
python3 seed.py 6 > seed.loon && $L2V seed.loon > seed_raw.vcad && python3 pose_seed.py seed_raw.vcad seed.vcad 1 || exit 1
time ./build_seed.sh
$R seed.vcad --view top --dims --size 1800 -o seed_top.png 2>/dev/null &
$R seed.vcad --azimuth 240 --elevation 18 --size 1600 --focus bench_2_frame -o seed_build.png 2>/dev/null &
wait
time python3 seed_video.py
