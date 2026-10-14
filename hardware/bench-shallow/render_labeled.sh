#!/bin/zsh
# usage: render_labeled.sh in.vcad out.png [vcad-render view args...]  -- labels from label_doc.py composited over a full render
R=${VCAD_RENDER:-vcad-render}
in=$1; out=$2; shift 2
t=${out:r}.tmp; mkdir -p $t
python3 ${0:A:h}/label_doc.py $in $t/nolinks.vcad
$R $in "$@" -o $t/full.png 2>/dev/null &
$R $t/nolinks.vcad "$@" --labels -o $t/lab.png 2>/dev/null &
$R $t/nolinks.vcad "$@" -o $t/plain.png 2>/dev/null &
wait
magick $t/lab.png $t/plain.png -alpha off -compose difference -composite -separate -evaluate-sequence max -threshold 0 -morphology dilate disk:1 $t/mask.png
magick $t/full.png $t/lab.png $t/mask.png -composite -background white -flatten $out
rm -rf $t
