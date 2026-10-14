#!/bin/zsh
# v2 rebuild: loon -> vcad -> renders + cut list + fit checks.  usage: ./build.sh
cd ${0:A:h}
R=${VCAD_RENDER:-vcad-render}
L2V=${LOON2VCAD:-loon2vcad}
set -e
mk() { # name args...   -> name.loon + name.vcad  (a *_lab doc also gets a *_lab_bb twin: bbox only)
  local n=$1; shift; python3 gen_rack.py "$@" > $n.loon; $L2V $n.loon > $n.vcad
  [[ $n == *_lab ]] && { python3 gen_rack.py "$@" --bbonly > ${n}_bb.loon; $L2V ${n}_bb.loon > ${n}_bb.vcad; } || true; }
mk rack_6u   --units 6;   mk rack_6u_lab  --units 6  --mode labels
mk rack_9u   --units 9;   mk rack_9u_lab  --units 9  --mode labels
mk rack_12u  --units 12;  mk rack_12u_lab --units 12 --mode labels
mk blade --mode blade;                 mk blade_lab --mode blade_labels
mk blade_exploded --mode blade_explode; mk blade_exploded_lab --mode blade_explode_labels
mk blade_chassis --mode chassis;       mk blade_chassis_lab --mode chassis_labels
mk gpu_node --mode gpu;                mk gpu_node_lab --mode gpu_labels
mk gpu_node_airflow --mode gpu_section
mk humanoid_bay --mode bay;            mk humanoid_bay_lab --mode bay_labels
{ for u in 6 9 12; do python3 gen_rack.py --units $u --mode cutlist; echo; done
  python3 gen_rack.py --mode checks; } > cutlist.txt

flat() { magick $1 -background "#0b0907" -flatten $1; }   # dark-theme PNGs are transparent
lab() { # full.vcad lab.vcad out.png args...  (label layer from a tags-only doc with the same bbox)
  local f=$1 l=$2 o=$3; shift 3; local t=${o:r}.tmp; mkdir -p $t
  $R $f "$@" -o $t/full.png 2>/dev/null
  $R $l "$@" --labels -o $t/lab.png 2>/dev/null
  $R $l "$@" -o $t/plain.png 2>/dev/null
  $R ${l:r}_bb.vcad "$@" --labels -o $t/bbl.png 2>/dev/null
  dm() { magick $1 $2 -alpha off -compose difference -composite -separate -evaluate-sequence max -threshold 0 -morphology dilate disk:2 $3; }
  dm $t/lab.png $t/plain.png $t/m1.png; magick $t/m1.png -morphology close disk:6 $t/m1.png; dm $t/bbl.png $t/plain.png $t/m2.png   # m2 = the bbox's own empty-label leader
  magick $t/m1.png $t/m2.png -compose minus_src -composite $t/mask.png
  magick $t/full.png $t/lab.png $t/mask.png -composite $o; rm -rf $t; flat $o; }
one() { local o=$1; shift; $R "$@" -o $o 2>/dev/null; flat $o; magick $o -trim +repage -bordercolor "#0b0907" -border 80 $o; }
D=(--theme dark --style tinted)
lab rack_9u.vcad  rack_9u_lab.vcad  rack_v2_iso.png  $D --azimuth 215 --elevation 25 --size 1600 &
lab rack_6u.vcad  rack_6u_lab.vcad  rack_6u_iso.png  $D --azimuth 215 --elevation 25 --size 1400 &
lab rack_12u.vcad rack_12u_lab.vcad rack_12u_iso.png $D --azimuth 215 --elevation 25 --size 1600 &
one rack_v2_front.png rack_9u.vcad $D --view front --dims --size 1400 &
one rack_v2_side.png  rack_9u.vcad $D --view side --section x=105 --dims --size 1600 &
lab blade_chassis.vcad blade_chassis_lab.vcad blade_chassis_iso.png $D --azimuth 215 --elevation 30 --size 1600 &
lab blade.vcad blade_lab.vcad blade_iso.png $D --azimuth 325 --elevation 20 --size 1400 &
lab blade_exploded.vcad blade_exploded_lab.vcad blade_exploded.png $D --azimuth 325 --elevation 20 --size 1600 &
lab gpu_node.vcad gpu_node_lab.vcad gpu_node_iso.png $D --azimuth 215 --elevation 30 --size 1600 &
one gpu_node_section.png gpu_node_airflow.vcad $D --view side --section x=105 --size 1600 &
lab humanoid_bay.vcad humanoid_bay_lab.vcad humanoid_bay_iso.png $D --azimuth 215 --elevation 25 --size 1400 &
wait
