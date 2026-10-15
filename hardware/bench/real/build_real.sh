#!/bin/zsh
# Rebuild every cell_real artifact: loon -> vcad -> posed docs -> stills.
cd ${0:A:h}
R=${VCAD_RENDER:-vcad-render}
python3 gen_cell_real.py > cell_real.loon || exit 1
${LOON2VCAD:-loon2vcad} cell_real.loon > cell_real_raw.vcad || exit 1
python3 make_frames_real.py pose TUCK 500 cell_real.vcad
python3 make_frames_real.py pose MILL 230 cell_real_mill.vcad
python3 make_frames_real.py pose OVEN 720 cell_real_oven.vcad
python3 make_frames_real.py pose PLATE 500 cell_real_plate.vcad
./render_labeled.sh cell_real.vcad cell_real_iso.png --azimuth 220 --elevation 28 --size 1600
./render_labeled.sh cell_real.vcad cell_real_top.png --view top --dims --size 1600
$R cell_real_mill.vcad --azimuth 235 --elevation 15 --size 1400 -o cell_real_mill.png 2>/dev/null
$R cell_real_oven.vcad --azimuth 300 --elevation 12 --size 1400 -o cell_real_oven.png 2>/dev/null
