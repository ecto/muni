.PHONY: all clean watch

all: main.pdf ../web/whitepaper.pdf

main.pdf: main.typ refs.bib $(wildcard images/*)
	typst compile main.typ

../web/whitepaper.pdf: main.pdf
	cp main.pdf ../web/whitepaper.pdf

watch:
	typst watch main.typ

clean:
	rm -f main.pdf
