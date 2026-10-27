#!/bin/zsh
cd ${0:A:h}
R=${VCAD_RENDER:-vcad-render}
./labeled.sh bay_one.vcad bay_iso.png "datum_l,datum_r,coupler,busbar,spine_rail_a:rail_a,spine_rail_b:rail_b,tender_plinth:plinth,plug,bay_floor:floor,slots" "" --azimuth 225 --elevation 30 --size 1600 &
./labeled.sh bay_faceonly.vcad bay_face.png "48V,CAN,air,coupler" "" --azimuth 270 --elevation 5 --size 1200 --focus coupler &
$R bay_empty.vcad --view top --dims --size 1600 -o bay_top.png 2>/dev/null &
./labeled.sh bay.vcad bay_bench.png "b3_coupler:bench coupler mates here" "a bench is three bays" --azimuth 215 --elevation 28 --size 1600 &
wait
