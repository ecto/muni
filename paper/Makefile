.PHONY: all clean watch watch-whitepaper watch-manual

# All documents
DOCS = whitepaper.pdf bvr0-manual.pdf bvr0-datasheet.pdf

all: $(DOCS) ../web/whitepaper.pdf ../web/bvr0-manual.pdf ../web/bvr0-datasheet.pdf

# Whitepaper
whitepaper.pdf: whitepaper.typ refs.bib lib/template.typ $(wildcard images/*)
	typst compile whitepaper.typ

# BVR0 Manual
bvr0-manual.pdf: bvr0-manual.typ lib/template.typ $(wildcard images/*)
	typst compile bvr0-manual.typ

# BVR0 Datasheet
bvr0-datasheet.pdf: bvr0-datasheet.typ lib/template.typ
	typst compile bvr0-datasheet.typ

# Copy documents to web folder
../web/whitepaper.pdf: whitepaper.pdf
	cp whitepaper.pdf ../web/whitepaper.pdf

../web/bvr0-manual.pdf: bvr0-manual.pdf
	cp bvr0-manual.pdf ../web/bvr0-manual.pdf

../web/bvr0-datasheet.pdf: bvr0-datasheet.pdf
	cp bvr0-datasheet.pdf ../web/bvr0-datasheet.pdf

# Watch modes
watch: watch-whitepaper

watch-whitepaper:
	typst watch whitepaper.typ

watch-manual:
	typst watch bvr0-manual.typ

clean:
	rm -f $(DOCS)
