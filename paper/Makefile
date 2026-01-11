.PHONY: all clean watch watch-whitepaper watch-manual watch-manual-bvr1 watch-deck watch-onepager investor web-docs

# All documents
DOCS = whitepaper.pdf bvr0-manual.pdf bvr1-manual.pdf bvr0-datasheet.pdf
INVESTOR_DOCS = pitch-deck.pdf one-pager.pdf
WEB_DOCS_DIR = ../web/public/docs
WEB_DOCS = $(WEB_DOCS_DIR)/whitepaper.pdf $(WEB_DOCS_DIR)/bvr0-manual.pdf $(WEB_DOCS_DIR)/bvr1-manual.pdf $(WEB_DOCS_DIR)/bvr0-datasheet.pdf
WEB_INVESTOR_DOCS = $(WEB_DOCS_DIR)/pitch-deck.pdf $(WEB_DOCS_DIR)/one-pager.pdf

# Font path for Berkeley Mono
TYPST = typst compile --font-path fonts/
TYPST_WATCH = typst watch --font-path fonts/

all: $(DOCS) $(WEB_DOCS)

# Investor documents
investor: $(INVESTOR_DOCS) $(WEB_INVESTOR_DOCS)

# Build + copy all docs to the website
web-docs: $(DOCS) $(INVESTOR_DOCS) $(WEB_DOCS) $(WEB_INVESTOR_DOCS)

# Whitepaper
whitepaper.pdf: whitepaper.typ refs.bib lib/template.typ $(wildcard images/*)
	$(TYPST) whitepaper.typ

# BVR0 Manual
bvr0-manual.pdf: bvr0-manual.typ lib/template.typ lib/diagrams.typ $(wildcard manual/*.typ) $(wildcard images/*)
	$(TYPST) bvr0-manual.typ

# BVR1 Manual
bvr1-manual.pdf: bvr1-manual.typ lib/template.typ lib/diagrams.typ $(wildcard manual/*.typ) $(wildcard manual-bvr1/*.typ) $(wildcard images/*)
	$(TYPST) bvr1-manual.typ

# BVR0 Datasheet
bvr0-datasheet.pdf: bvr0-datasheet.typ lib/template.typ
	$(TYPST) bvr0-datasheet.typ

# Pitch Deck
pitch-deck.pdf: pitch-deck.typ lib/template.typ
	$(TYPST) pitch-deck.typ

# One-Pager
one-pager.pdf: one-pager.typ lib/template.typ
	$(TYPST) one-pager.typ

# Copy documents to web folder
$(WEB_DOCS_DIR):
	mkdir -p $(WEB_DOCS_DIR)

$(WEB_DOCS_DIR)/whitepaper.pdf: whitepaper.pdf | $(WEB_DOCS_DIR)
	cp $< $@

$(WEB_DOCS_DIR)/bvr0-manual.pdf: bvr0-manual.pdf | $(WEB_DOCS_DIR)
	cp $< $@

$(WEB_DOCS_DIR)/bvr1-manual.pdf: bvr1-manual.pdf | $(WEB_DOCS_DIR)
	cp $< $@

$(WEB_DOCS_DIR)/bvr0-datasheet.pdf: bvr0-datasheet.pdf | $(WEB_DOCS_DIR)
	cp $< $@

$(WEB_DOCS_DIR)/pitch-deck.pdf: pitch-deck.pdf | $(WEB_DOCS_DIR)
	cp $< $@

$(WEB_DOCS_DIR)/one-pager.pdf: one-pager.pdf | $(WEB_DOCS_DIR)
	cp $< $@

# Watch modes
watch: watch-whitepaper

watch-whitepaper:
	$(TYPST_WATCH) whitepaper.typ

watch-manual:
	$(TYPST_WATCH) bvr0-manual.typ

watch-manual-bvr1:
	$(TYPST_WATCH) bvr1-manual.typ

watch-deck:
	$(TYPST_WATCH) pitch-deck.typ

watch-onepager:
	$(TYPST_WATCH) one-pager.typ

clean:
	rm -f $(DOCS) $(INVESTOR_DOCS)
