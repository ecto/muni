.PHONY: all clean watch png darken

# Fonts: IBM Plex Sans + Mono (OFL 1.1, fonts/OFL-IBM-Plex.txt). System fonts ignored so builds are reproducible.
# Paths are relative to this Makefile, so `make` works from paper/vision/.
HERE := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
TYPST ?= typst
TYPST_C = $(TYPST) compile --root $(HERE) --font-path $(HERE)fonts --ignore-system-fonts
TYPST_W = $(TYPST) watch --root $(HERE) --font-path $(HERE)fonts --ignore-system-fonts

all: vision.pdf

# darken.py needs numpy + Pillow; override with PYTHON=/path/to/python if uv is unavailable.
PYTHON ?= uv run --offline --quiet --no-project --with numpy --with pillow python

vision.pdf: vision.typ refs.bib $(wildcard images-dark/*.png) $(wildcard images-native/*.png) $(wildcard fonts/*)
	cd $(HERE) && $(TYPST_C) vision.typ

# Light renders in images/ are re-toned onto the page palette in images-dark/.
# The re-toned output is committed, so this is only needed when images/ changes.
darken:
	cd $(HERE) && $(PYTHON) darken.py

png: vision.typ
	cd $(HERE) && mkdir -p out && rm -f out/p*.png && $(TYPST_C) --ppi 110 vision.typ 'out/p{0p}.png'

watch:
	cd $(HERE) && $(TYPST_W) vision.typ

clean:
	rm -f vision.pdf
