VERSION := $(shell node -e "console.log(require('./package.json').version);")
ROLLUP := node_modules/.bin/rollup

default: jpv-$(VERSION).zip jpv-$(VERSION).xpi jpv-$(VERSION)-source.zip

.PHONY: chrome firefox

chrome:
	ln -fs manifests/manifest-chrome.json manifest.json

firefox:
	ln -fs manifests/manifest-ff.json manifest.json

node_modules:
	npm install

icons/jpv-%.png: graphics/jpv.png
	magick $? -resize $*x$* $@

icons/jpv-disabled-%.png: graphics/jpv-disabled.png
	magick $? -resize $*x$* $@

ICONS := icons/jpv-19.png icons/jpv-38.png icons/jpv-48.png icons/jpv-64.png icons/jpv-128.png icons/jpv-256.png
ICONS += icons/jpv-disabled-19.png icons/jpv-disabled-38.png icons/jpv-disabled-48.png icons/jpv-disabled-64.png icons/jpv-disabled-128.png icons/jpv-disabled-256.png

content_scripts/index.js: node_modules lib/*.ts content_scripts/*.ts
	${ROLLUP} --format cjs -p typescript -i content_scripts/index.ts -o $@

pages/popup.js: node_modules lib/*.ts pages/*.ts
	${ROLLUP} --format cjs -p typescript -i pages/popup.ts -o $@

pages/options.js: node_modules lib/*.ts pages/*.ts
	${ROLLUP} --format cjs -p typescript -i pages/options.ts -o $@

background/background.js: node_modules lib/*.ts background/*.ts
	${ROLLUP} --format cjs -p typescript -i background/background.ts -o $@

BUILD := manifest.json
BUILD += content_scripts/index.js content_scripts/index.css
BUILD += pages/popup.js pages/popup.html
BUILD += pages/options.js pages/options.html
BUILD += pages/pages.css
BUILD += background/background.js
BUILD += $(ICONS)

jpv-$(VERSION).zip: ${BUILD}
	zip -1 $@ $?

jpv-$(VERSION).xpi: ${BUILD}
	zip -1 $@ $?

SOURCES := Makefile README.md tsconfig.json manifest.json package.json package-lock.json
SOURCES += content_scripts/*.ts content_scripts/index.css
SOURCES += pages/*.ts pages/*.css pages/*.html
SOURCES += background/*.ts
SOURCES += lib/*.ts
SOURCES += $(ICONS)

jpv-$(VERSION)-source.zip: ${SOURCES}
	zip -1 $@ $?

clean:
	rm -f icons/*.png
	rm -rf out
	rm -f pages/*.js
	rm -f content_scripts/index.js
	rm -f background/background.js
	rm -f jpv-*.zip
	rm -f jpv-*.xpi
	rm -f jpv-*-source.zip
