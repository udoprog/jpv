VERSION := $(shell node -e "console.log(require('./package.json').version);")

default: jpv-$(VERSION).xpi

node_modules:
	npm install

jpv.js: node_modules
	npm run build

jpv-$(VERSION).xpi: jpv.js jpv.css manifest.json icons/*
	zip -1 $@ $?

clean:
	rm -rf out
	rm jpv-*.xpi