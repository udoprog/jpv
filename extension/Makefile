VERSION := $(shell node -e "console.log(require('./package.json').version);")
ROLLUP := ${PWD}/node_modules/.bin/rollup

default: jpv-$(VERSION).xpi jpv-$(VERSION)-source.zip

node_modules:
	npm install

content_scripts/index.js: node_modules lib/*.ts content_scripts/*.ts
	${ROLLUP} --format cjs -p typescript -i content_scripts/index.ts -o $@

popup/popup.js: node_modules lib/*.ts popup/*.ts
	${ROLLUP} --format cjs -p typescript -i popup/popup.ts -o $@

background/background.js: node_modules lib/*.ts background/*.ts
	${ROLLUP} --format cjs -p typescript -i background/background.ts -o $@

BUILD := manifest.json icons/*
BUILD += content_scripts/index.js content_scripts/index.css
BUILD += popup/popup.js popup/popup.css popup/popup.html
BUILD += background/background.js

jpv-$(VERSION).xpi: ${BUILD}
	zip -1 $@ $?

SOURCES := Makefile README.md tsconfig.json manifest.json package.json package-lock.json icons/*
SOURCES += content_scripts/*.ts content_scripts/index.css
SOURCES += popup/*.ts popup/popup.css popup/popup.html
SOURCES += background/*.ts
SOURCES += lib/*.ts

jpv-$(VERSION)-source.zip: ${SOURCES}
	zip -1 $@ $?

clean:
	rm -rf out
	rm -f content_scripts/index.js
	rm -f popup/popup.js
	rm -f background/background.js
	rm -f jpv-*.xpi
	rm -f jpv-*-source.zip
