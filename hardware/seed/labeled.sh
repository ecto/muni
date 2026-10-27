#!/bin/zsh
# usage: labeled.sh in.vcad out.png "keep1,keep2,..." [rename frame=NAME] -- [vcad-render args]
# Labels come from a copy holding only the listed instances (+ the bench frame, for identical framing), composited
# over a label-free render of the full doc (vcad-render --labels otherwise tags every arm link).
R=${VCAD_RENDER:-vcad-render}
in=$1; out=$2; keep=$3; fname=$4; shift 4
t=${out:r}.tmp; mkdir -p $t
python3 - $in $t/lab.vcad "$keep" "$fname" <<'PY'
import json, sys
d = json.load(open(sys.argv[1])); kv = [p.split(':') for p in sys.argv[3].split(',')]; ren = {p[0]: p[-1] for p in kv}; keep = set(ren); fn = sys.argv[4]
d['instances'] = [i for i in d['instances'] if i['id'] in keep or i['id'] == 'frame']
d['joints'] = [j for j in d['joints'] if j['childInstanceId'] in keep]
for i in d['instances']:
    if i['id'] == 'frame': i['name'] = fn
    elif i['id'] in ren: i['name'] = ren[i['id']]
json.dump(d, open(sys.argv[2], 'w'))
PY
$R $in "$@" -o $t/full.png 2>/dev/null &
$R $t/lab.vcad "$@" --labels -o $t/lab.png 2>/dev/null &
$R $t/lab.vcad "$@" -o $t/plain.png 2>/dev/null &
wait
magick $t/lab.png $t/plain.png -alpha off -compose difference -composite -separate -evaluate-sequence max -threshold 0 -morphology dilate disk:1 $t/mask.png
magick $t/full.png $t/lab.png $t/mask.png -composite $out
rm -rf $t
