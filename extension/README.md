Browser extension to enable fast shift+hovering over text to quickly perform
dictionary lookups.

![Shift+hovering over text to quickly perform a dictionary lookup](screencap.png)

## Building

Building depends on:
* `make`.
* A recent version of `node`.
* An operating system that can run both of those or WSL.

To just build the extension, run:

```
npm install
npm run build
```

To build and package:

```
make
```

## Developing

To build the project for in-place, you can watch the project for changes.

```
npm run watch
```
