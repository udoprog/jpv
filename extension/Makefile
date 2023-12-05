VERSION := $(shell node -e "console.log(require('./package.json').version);")

default: jpv-$(VERSION).xpi jpv-$(VERSION)-source.zip

node_modules:
	npm install

jpv.js: node_modules src/*.ts
	npm run build

jpv-$(VERSION).xpi: jpv.js jpv.css manifest.json icons/*
	zip -1 $@ $?

jpv-$(VERSION)-source.zip: Makefile README.md tsconfig.json jpv.css manifest.json package.json package-lock.json icons/* src/*.ts
	zip -1 $@ $?

clean:
	rm -rf out
	rm -f jpv.js
	rm -f jpv-$(VERSION).xpi
	rm -f jpv-$(VERSION)-source.zip
