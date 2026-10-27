#!/bin/zsh
cd ${0:A:h}
R=${VCAD_RENDER:-vcad-render}
./labeled.sh seed.vcad seed_iso.png "regolith: ,genome,bought_parts,collector,battery,hopper,sinter,bench_2_frame,scale_1m" "bench" --azimuth 210 --elevation 22 --size 1800 &
wait
magick seed_iso.png -trim +repage -bordercolor white -border 40 seed_iso.png
