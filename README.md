# jpv

This is my personal dictionary project.

To use, you'll have to download
- `JMdict_e_examp.gz` from <http://www.edrdg.org/wiki/index.php/Main_Page>
- `kanjidic2.xml.gz` from <http://www.edrdg.org/wiki/index.php/KANJIDIC_Project>
and place them in the root of the repository, and then run:

```sh
RUST_LOG="lib=info" cargo run --release -p tools --bin build-database
```

After that, install trunk and build the web-ui:

```sh
cargo install trunk
cargo toolchain add wasm32-unknown-unknown
trunk build --release
```

Now you can run the bundled web-ui:

```
cargo run --release -p jpv --features bundle
```

![Good morning!](splash.png)
