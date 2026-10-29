#!/bin/bash
# each phase takes the shared GPU lock and releases it after
cd ~/gpsm-bench
B="python3 -E -P scripts/bench.py"
L="flock $HOME/tang-gpu.lock"
ph() { echo "=== $(date +%T) $*" >> logs/drive.log; $L $B "$@" >> logs/run.log 2>&1 || echo "FAIL $*" >> logs/drive.log; }
ph gpsm-write facts facts
ph gpsm-ask facts 60
for a in A0 A1 A2 A3; do ph base $a facts; done
ph gpsm-write-all locomo
for a in A0 A1 A2 A3; do ph base $a locomo; done
for k in 1 2; do ph gpsm-ask locomo 125; done
echo "=== $(date +%T) ALL DONE" >> logs/drive.log
exit 0
