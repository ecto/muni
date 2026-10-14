#!/bin/zsh
# rebuild: gen.py -> loon -> vcad -> dark renders, reach check.   usage: ./build.sh
cd ${0:A:h}
R=${VCAD_RENDER:-vcad-render}
L2V=${LOON2VCAD:-loon2vcad}
set -e
mk() { # name mode  -> name.loon + name.vcad  (a *_lab doc also gets a *_lab_bb twin: bbox only)
  local n=$1 m=$2; python3 gen.py --mode $m > $n.loon; $L2V $n.loon > $n.vcad
  [[ $n == *_lab ]] && { python3 gen.py --mode $m --bbonly > ${n}_bb.loon; $L2V ${n}_bb.loon > ${n}_bb.vcad; } || true; }
mk pack pack;               mk pack_lab pack_labels
mk pack_exploded explode;   mk pack_exploded_lab explode_labels
mk pack_section section
mk receiver receiver;       mk receiver_lab receiver_labels
for i in 1 2 3 4; do mk swap_$i swap$i; done
mk dock dock;               mk dock_lab dock_labels
python3 reach.py > reach.txt

flat() { magick $1 -background "#0b0907" -flatten $1; }
lab() { # full.vcad lab.vcad out.png args...   (same label-compositing trick as ../compute/build.sh)
  local f=$1 l=$2 o=$3; shift 3; local t=${o:r}.tmp; mkdir -p $t
  $R $f "$@" -o $t/full.png 2>/dev/null
  $R $l "$@" --labels -o $t/lab.png 2>/dev/null
  $R $l "$@" -o $t/plain.png 2>/dev/null
  $R ${l:r}_bb.vcad "$@" --labels -o $t/bbl.png 2>/dev/null
  dm() { magick $1 $2 -alpha off -compose difference -composite -separate -evaluate-sequence max -threshold 0 -morphology dilate disk:2 $3; }
  dm $t/lab.png $t/plain.png $t/m1.png; magick $t/m1.png -morphology close disk:6 $t/m1.png; dm $t/bbl.png $t/plain.png $t/m2.png
  magick $t/m1.png $t/m2.png -compose minus_src -composite $t/mask.png
  magick $t/full.png $t/lab.png $t/mask.png -composite $o; rm -rf $t; flat $o; }
one() { local o=$1; shift; $R "$@" -o $o 2>/dev/null; flat $o; magick $o -trim +repage -bordercolor "#0b0907" -border 60 $o; }
D=(--theme dark --style tinted)
lab pack.vcad pack_lab.vcad pack_iso.png $D --azimuth 300 --elevation 22 --size 1400 &
lab pack_exploded.vcad pack_exploded_lab.vcad pack_exploded.png $D --azimuth 300 --elevation 22 --size 1600 &
one pack_section.png pack_section.vcad $D --view top --section z=120 --size 1200 &
lab receiver.vcad receiver_lab.vcad robot_receiver_iso.png $D --azimuth 240 --elevation 25 --size 1500 &
for i in 1 2 3 4; do one swap_$i.png swap_$i.vcad $D --azimuth 290 --elevation 12 --size 1300 & done
lab dock.vcad dock_lab.vcad dock_iso.png $D --azimuth 240 --elevation 50 --size 1400 &
wait
