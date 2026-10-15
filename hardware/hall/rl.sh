#!/bin/zsh
# rl.sh full.vcad lab.vcad out.png [render args]
R=${VCAD_RENDER:-vcad-render}
f=$1; l=$2; out=$3; shift 3; t=${out:r}.tmp; mkdir -p $t
$R $f "$@" -o $t/full.png 2>/dev/null
$R $l "$@" --labels -o $t/lab.png 2>/dev/null
$R $l "$@" -o $t/plain.png 2>/dev/null
magick $t/lab.png $t/plain.png -alpha off -compose difference -composite -separate -evaluate-sequence max -threshold 0 -morphology dilate disk:1 $t/mask.png
magick \( $t/full.png -background white -alpha remove \) $t/lab.png $t/mask.png -composite $out
rm -rf $t
