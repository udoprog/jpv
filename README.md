# jpv

<a href="https://github.com/udoprog/jpv">
<img height="128" width="128" alt="Japanese Dictionary by John-John Tedro" src="https://github.com/udoprog/jpv/blob/main/gfx/logo.png?raw=true" />
</a>

Welcome to my Japanese dictionary project!

This used to be a personal project of mine, but I have now spent enough time and
effort on it that I think it might be useful for others.

<br>

## Overview

<table>
<tr>
<td valign="top">
  <img alt="Searching for english text" src="https://github.com/udoprog/jpv/blob/main/gfx/feature-english.png?raw=true" />
  <div style="font-size: 0.8em;">Search for Japanese words and phrases or English glossary.</div>
</td>
<td valign="top">
  <img alt="Conjugations" src="https://github.com/udoprog/jpv/blob/main/gfx/feature-conjugate.png?raw=true" /><br>
  <div style="font-size: 0.8em;">Advanced word conjugator.</div>
</td>
</tr>

<tr>
<td valign="top">
  <img alt="Image recognition using tesseract through the clipboard" src="https://github.com/udoprog/jpv/blob/main/gfx/feature-ocr.png?raw=true" />
  <div style="font-size: 0.8em;">Image recognition through the clipboard using <a href="https://github.com/tesseract-ocr/tesseract">tesseract</a> (<code>ocr</code> feature).</div>
</td>
<td valign="top">
  <img alt="Wildcard searching" src="https://github.com/udoprog/jpv/blob/main/gfx/feature-wildcard.png?raw=true" />
  <div style="font-size: 0.8em;">Wildcard searching.</div>
</td>
</tr>
</table>

<br>

## Building and Installing

Install dependencies for the platform you intend to build for:

* For the `gnome` feature:
  * `Fedora` - `sudo dnf install dbus-devel pkgconf-pkg-config`
* For the `ocr` feature:
  * `Fedora` - `sudo dnf install tesseract-devel`

Install [`trunk`] and the `wasm32` toolchain to build the UI:

[`trunk`]: https://trunkrs.dev/

```sh
cargo install trunk
cargo toolchain add wasm32-unknown-unknown
```

After this, you can run the project directly in the project directory:

```sh
trunk build --release
cargo run --features bundle,gnome
```

There are scripts available to conveniently build and install packages for
specific environments:

* [tools/install-fedora](tools/install-fedora) to build and install for GNOME on
  Fedora.

You can also the project manually, but this will lack any system integration
like clipboard capture:

```
cargo install --path crates/jpv --features bundle,gnome
```

<br>

## Configuring

After `jpv` has been installed, you must construct the dictionary file the
project will use.

```
jpv build
```

After this, you can start the dictionary in the background with. This will also
automatically open up the interface.

```
jpv service --background
```

![Good morning!](https://github.com/udoprog/jpv/blob/main/gfx/splash.png?raw=true)

<br>

## Features

For rust features, we have the following:

* The `gnome` feature enabled full GNOME desktop integration, which includes the
  `dbus` and `ocr` features. This is also necessary to use the GNOME extension
  to capture the clipboard.
* The `dbus` feature provides the ability for the service to interact with
  D-Bus. Which is necessary for extensions to communicate with it and to perform
  D-Bus activation.
* The `ocr` feature provides image recognition for clipboard events where the
  mimetype is appropriate.
* The `mmap` feature (Unix only) loads the database using memory maps.

<br>

#### Interface

The dictionary is primarily interacted with using the `jpv` tools. It has a
comprehensive help section you can get through `jpv --help`, but some of the
more notable features are:

* `jpv cli <query>` can be used to perform commandline queries.
* `jpv send-clipboard --type text/plain hello` can be used to inject a phrase
  into the dictionary for analysis (requires the `dbus` feature).

All relevant tools that interact with the background service rely on features
such as D-Bus activation, which will ensure that a background service is up and
running as needed.

<br>

## Building and packing for Fedora GNOME

To build an rpm package which is suitable for Fedora GNOME, you can do the following:

```sh
cargo build --release -p jpv --features bundle,gnome
cargo generate-rpm -p crates/jpv
```

The generated rpm will be located in `target/generate-rpm`.

```sh
sudo npm -i target/generate-rpm/jpv-0.0.0-1.x86_64.rpm
```

Once complete, this installs a desktop entry you can use to start the dictionary
in the background. Starting the application will open up the browser UI.

Note that you still need to build the database before it can be used.

![Desktop entry](https://github.com/udoprog/jpv/blob/main/gfx/desktop.png?raw=true)

<br>

#### GNOME Extension

Since GNOME and Wayland desktop environments in general currently do not have
any facilities to generically capture the clipboard we must rely on extensions.

To enable the Japanese Dictionary extension for gnome, start the extensions
manager after installing the package:

![Gnome extension](https://github.com/udoprog/jpv/blob/main/gfx/gnome-extension.png?raw=true)

Once enabled, clipboard capture has to be enabled in the panel item.

<table>
<tr>
<td valign="top">
  <img alt="Searching for english text" src="https://github.com/udoprog/jpv/blob/main/gfx/gnome-clipboard-capture.png?raw=true" />
  <div style="font-size: 0.8em;">Extension button.</div>
</td>
<td valign="top">
  <img alt="Conjugations" src="https://github.com/udoprog/jpv/blob/main/gfx/gnome-clipboard-capture-enabled.png?raw=true" /><br>
  <div style="font-size: 0.8em;">Clipboard capture enabled.</div>
</td>
</tr>
</table>

> **Note:** while clipboard capture is running the extension icon will be red.
> Only enable it while it's in use since there are currently no security
> mechanisms in place other than your local system. Any application could
> pretend to be a dictionary application and capture the clipboard.

Clipboard capture is governed by the `capture-clipboard-enabled` setting:

```sh
> gsettings get se.tedro.japanese-dictionary.plugins capture-clipboard-enabled
true
> gsettings set se.tedro.japanese-dictionary.plugins capture-clipboard-enabled false
```
