#!/bin/zsh
# Rebuild everything: ./build.sh   (loon -> vcad -> posed docs -> stills -> video/gif -> numbers)
cd ${0:A:h}
R=${VCAD_RENDER:-vcad-render}
L2V=${LOON2VCAD:-loon2vcad}
FF=/opt/homebrew/bin/ffmpeg
set -e
python3 gen_gripper.py --tips flat > gripper.loon && $L2V gripper.loon > gripper_raw.vcad
python3 gen_gripper.py --tips flat --can > gripper_can.loon && $L2V gripper_can.loon > gripper_can_raw.vcad
python3 gen_gripper.py --tips soft --boot > gripper_rover.loon && $L2V gripper_rover.loon > gripper_rover_raw.vcad
python3 gen_gripper.py --display > gripper_tips.loon && $L2V gripper_tips.loon > gripper_tips.vcad
python3 gen_gripper.py --tips flat --adapter > gripper_adapter.loon && $L2V gripper_adapter.loon > gripper_adapter_raw.vcad
python3 pose.py gripper_raw.vcad pose_open.vcad 100
STRIP=1 python3 pose.py gripper_raw.vcad lbl_open.vcad 100
STRIP=1 python3 pose.py gripper_can_raw.vcad lbl_can.vcad 66
STRIP=1 python3 pose.py gripper_rover_raw.vcad lbl_rover.vcad 60
python3 pose.py gripper_can_raw.vcad pose_can.vcad 66
python3 pose.py gripper_rover_raw.vcad pose_rover.vcad 60
STRIP=1 python3 pose.py gripper_adapter_raw.vcad lbl_adapter.vcad 60
$R lbl_open.vcad --azimuth 215 --elevation 25 --size 1600 --labels -o gripper_iso_open.png 2>/dev/null &
$R lbl_can.vcad --azimuth 215 --elevation 20 --size 1600 --labels -o gripper_iso_closed.png 2>/dev/null &
$R gripper_tips.vcad --azimuth 240 --elevation 35 --size 1600 --labels -o gripper_tips.png 2>/dev/null &
$R lbl_open.vcad --view front --section y=0 --size 1600 --labels -o gripper_section.png 2>/dev/null &
$R pose_open.vcad --view front --dims --size 1600 -o gripper_front.png 2>/dev/null &
$R lbl_rover.vcad --azimuth 215 --elevation 25 --size 1600 --labels -o gripper_rover.png 2>/dev/null &
$R pose_open.vcad --view top --dims --size 1600 -o gripper_flange.png 2>/dev/null &
$R lbl_adapter.vcad --azimuth 215 --elevation 25 --size 1600 --labels -o gripper_adapter.png 2>/dev/null &
wait
python3 numbers.py > numbers.txt
[[ -n $NOVIDEO ]] || python3 frames.py
[[ -n $NOVIDEO ]] || $FF -y -loglevel error -framerate 24 -i frames/%04d.png -c:v libx264 -pix_fmt yuv420p -crf 20 gripper_grasp.mp4
[[ -n $NOVIDEO ]] || $FF -y -loglevel error -i gripper_grasp.mp4 -vf "setpts=PTS/2,fps=12,scale=560:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=96[p];[b][p]paletteuse" gripper_grasp.gif
rm -rf frames
