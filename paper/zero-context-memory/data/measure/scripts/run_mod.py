import sys, os, runpy
REPO = os.path.expanduser("~/qwenputer-repro/repo/experimental/riscv")
sys.path.insert(0, REPO); os.chdir(REPO)
mod = sys.argv[1]; sys.argv = [mod] + sys.argv[2:]
runpy.run_module(mod, run_name="__main__")
