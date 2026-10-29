#!/bin/bash
cd ~/gpsm-measure
export BASE=$HOME/gpsm-models/Qwen3.5-4B FS=$HOME/gpsm-models/Qwen3.5-4B-Qwenputer-Filesystem
L=out/logs
while pgrep -u cam -f "speed.py fs_rvchat" > /dev/null; do sleep 10; done
echo "=== speed_fs_rvchat done(?) $(date)" >> $L/pipeline.log
step() { local t=$1 n=$2; shift 2; echo "=== $n $(date)" >> $L/pipeline.log; timeout $t "$@" > $L/$n.log 2>&1; echo "=== $n rc=$? $(date)" >> $L/pipeline.log; }
export -f step; export L
# group 1 (~20 min): fs_hf speed, KL base x3, KL fs x3
flock ~/tang-gpu.lock bash -c '
step 600 speed_fs_hf python3 -P speed.py fs_hf
step 300 kl_base_rust python3 -P kl_code.py base code/rust_itoa_lib.rs rust
step 300 kl_base_py python3 -P kl_code.py base code/py_textwrap.py python
step 300 kl_base_prose python3 -P kl_code.py base text.txt prose'
# group 2: capacity (makes full_small.npz), then fs KL
flock ~/tang-gpu.lock bash -c '
step 1800 capacity python3 -P capacity.py'
flock ~/tang-gpu.lock bash -c '
step 600 kl_fs_rust python3 -P kl_code.py fs code/rust_itoa_lib.rs rust
step 600 kl_fs_py python3 -P kl_code.py fs code/py_textwrap.py python
step 600 kl_fs_prose python3 -P kl_code.py fs text.txt prose'
for i in 1 2 3; do
 for d in dmx_mul_big dmx_div_big; do
  flock ~/tang-gpu.lock bash -c "
  step 1200 arith_rv1_${d}_$i python3 -P run_mod.py chipformer.rv.evalarith $HOME/gpsm-models/cpu --cond rv1 --data $d --n 40 --batch 4 --out $HOME/gpsm-measure/out/arith"
  flock ~/tang-gpu.lock bash -c "
  step 1200 arith_base_${d}_$i python3 -P run_mod.py chipformer.rv.evalarith $BASE --cond base_nudge --data $d --n 40 --batch 8 --out $HOME/gpsm-measure/out/arith"
 done
done
echo ALLDONE >> $L/pipeline.log
