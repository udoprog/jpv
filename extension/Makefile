VERSION := $(shell node -e "console.log(require('./package.json').version);")

default: jpv-$(VERSION).xpi jpv-$(VERSION)-source.zip

node_modules:
	npm install

content_scripts/index.min.js: node_modules lib/*.ts content_scripts/*.ts
	npm run build-content-scripts

popup/popup.min.js: node_modules lib/*.ts popup/*.ts
	npm run build-popup

BUILD := manifest.json icons/*
BUILD += content_scripts/index.min.js content_scripts/index.css
BUILD += popup/popup.min.js popup/popup.css popup/popup.html

jpv-$(VERSION).xpi: ${BUILD}
	zip -1 $@ $?

SOURCES := Makefile README.md tsconfig.json manifest.json package.json package-lock.json icons/*
SOURCES += content_scripts/*.ts content_scripts/index.css
SOURCES += popup/*.ts popup/popup.css
SOURCES += lib/*.ts

jpv-$(VERSION)-source.zip: ${SOURCES}
	zip -1 $@ $?

clean:
	rm -rf out
	rm -f content_scripts/index.min.js
	rm -f popup/popup.min.js
	rm -f jpv-*.xpi
	rm -f jpv-*-source.zip
