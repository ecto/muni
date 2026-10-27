#!/bin/zsh
# usage: render_labeled.sh in.vcad out.png [vcad-render view args...]
# vcad-render --labels labels EVERY instance (14 arm links = unreadable pile).
# Workaround: labels come from a copy with the arm/carriage instances removed,
# composited over a label-free render of the full posed doc (same framing:
# the arms sit inside the frame bbox).
R=${VCAD_RENDER:-vcad-render}
in=$1; out=$2; shift 2
t=${out:r}.tmp; mkdir -p $t
python3 ${0:A:h}/label_doc_v2.py $in $t/nolinks.vcad
$R $in "$@" -o $t/full.png 2>/dev/null
$R $t/nolinks.vcad "$@" --labels -o $t/lab.png 2>/dev/null
$R $t/nolinks.vcad "$@" -o $t/plain.png 2>/dev/null
# dims flag (if given) applies to all three renders identically, so it is kept in full.png too
magick $t/lab.png $t/plain.png -alpha off -compose difference -composite -separate -evaluate-sequence max -threshold 0 -morphology dilate disk:1 $t/mask.png
magick $t/full.png $t/lab.png $t/mask.png -composite $out
rm -rf $t
