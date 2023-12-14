VERSION := $(shell node -e "console.log(require('./package.json').version);")
ROLLUP := node_modules/.bin/rollup

default: jpv-$(VERSION).xpi jpv-$(VERSION)-source.zip

node_modules:
	npm install

content_scripts/index.js: node_modules lib/*.ts content_scripts/*.ts
	${ROLLUP} --format cjs -p typescript -i content_scripts/index.ts -o $@

pages/popup.js: node_modules lib/*.ts pages/*.ts
	${ROLLUP} --format cjs -p typescript -i pages/popup.ts -o $@

pages/options.js: node_modules lib/*.ts pages/*.ts
	${ROLLUP} --format cjs -p typescript -i pages/options.ts -o $@

background/background.js: node_modules lib/*.ts background/*.ts
	${ROLLUP} --format cjs -p typescript -i background/background.ts -o $@

BUILD := manifest.json icons/*
BUILD += content_scripts/index.js content_scripts/index.css
BUILD += pages/popup.js pages/popup.html
BUILD += pages/options.js pages/options.html
BUILD += pages/pages.css
BUILD += background/background.js

jpv-$(VERSION).xpi: ${BUILD}
	zip -1 $@ $?

SOURCES := Makefile README.md tsconfig.json manifest.json package.json package-lock.json icons/*
SOURCES += content_scripts/*.ts content_scripts/index.css
SOURCES += pages/*.ts pages/*.css pages/*.html
SOURCES += background/*.ts
SOURCES += lib/*.ts

jpv-$(VERSION)-source.zip: ${SOURCES}
	zip -1 $@ $?

clean:
	rm -rf out
	rm -f pages/*.js
	rm -f content_scripts/index.js
	rm -f background/background.js
	rm -f jpv-*.xpi
	rm -f jpv-*-source.zip
