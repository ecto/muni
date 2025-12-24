.PHONY: all clean watch watch-whitepaper watch-manual investor

# All documents
DOCS = whitepaper.pdf bvr0-manual.pdf bvr0-datasheet.pdf
INVESTOR_DOCS = pitch-deck.pdf one-pager.pdf

all: $(DOCS) ../web/whitepaper.pdf ../web/bvr0-manual.pdf ../web/bvr0-datasheet.pdf

# Investor documents
investor: $(INVESTOR_DOCS) ../web/pitch-deck.pdf

# Whitepaper
whitepaper.pdf: whitepaper.typ refs.bib lib/template.typ $(wildcard images/*)
	typst compile whitepaper.typ

# BVR0 Manual
bvr0-manual.pdf: bvr0-manual.typ lib/template.typ $(wildcard images/*)
	typst compile bvr0-manual.typ

# BVR0 Datasheet
bvr0-datasheet.pdf: bvr0-datasheet.typ lib/template.typ
	typst compile bvr0-datasheet.typ

# Pitch Deck
pitch-deck.pdf: pitch-deck.typ lib/template.typ
	typst compile pitch-deck.typ

# One-Pager
one-pager.pdf: one-pager.typ lib/template.typ
	typst compile one-pager.typ

# Copy documents to web folder
../web/whitepaper.pdf: whitepaper.pdf
	cp whitepaper.pdf ../web/whitepaper.pdf

../web/bvr0-manual.pdf: bvr0-manual.pdf
	cp bvr0-manual.pdf ../web/bvr0-manual.pdf

../web/bvr0-datasheet.pdf: bvr0-datasheet.pdf
	cp bvr0-datasheet.pdf ../web/bvr0-datasheet.pdf

../web/pitch-deck.pdf: pitch-deck.pdf
	cp pitch-deck.pdf ../web/pitch-deck.pdf

# Watch modes
watch: watch-whitepaper

watch-whitepaper:
	typst watch whitepaper.typ

watch-manual:
	typst watch bvr0-manual.typ

watch-deck:
	typst watch pitch-deck.typ

watch-onepager:
	typst watch one-pager.typ

clean:
	rm -f $(DOCS) $(INVESTOR_DOCS)
