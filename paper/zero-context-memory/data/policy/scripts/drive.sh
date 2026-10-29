#!/bin/bash
# each phase takes the shared GPU lock and releases it after; phase wall times go to logs/drive.log
cd ~/gpsm-bench/policy
ph() { echo "=== $(date +%T) start $*" >> logs/drive.log; s=$(date +%s)
  flock $HOME/tang-gpu.lock bash -c "echo \"--- \$(date +%T) lock $*\" >> logs/drive.log; t=\$(date +%s); $PFX python3 -E -P scripts/policy.py $* >> logs/run.log 2>&1 || echo FAIL >> logs/drive.log; echo \"--- gpu_seconds \$((\$(date +%s)-t)) $*\" >> logs/drive.log"; }
for p in "$@"; do ph $p; done
echo "=== $(date +%T) DONE $*" >> logs/drive.log
