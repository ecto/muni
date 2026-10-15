#!/bin/zsh
# Rebuild the hall: model + static renders (hall_top/iso/dock/carry), then growth stills + video.
cd ${0:A:h}
export PATH=/opt/homebrew/bin:$PATH
time python3 run_static.py
time python3 growth.py
