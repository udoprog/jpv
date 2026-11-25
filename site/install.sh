#!/bin/sh
# Installs the latest prebuilt jpv into ${JPV_INSTALL_DIR:-$HOME/.local/bin}.
#
#   curl -fsSL https://udoprog.github.io/jpv/install.sh | sh
#
# JPV_VERSION pins a release by tag (e.g. `nightly` or a dated tag) instead of
# the latest. If the latest release has no archive for this machine, the
# `nightly` release is used instead.
#
# The portable archives leave out OCR (it links tesseract). For OCR use the
# Fedora RPM, or build from source. Windows is not supported by this script;
# use the MSI from the releases.

set -eu

repo="udoprog/jpv"
api="https://api.github.com/repos/$repo/releases"

say() {
    printf '%s\n' "jpv-install: $*"
}

die() {
    printf '%s\n' "jpv-install: error: $*" >&2
    exit 1
}

# Prints the <arch>-<os> pair for this machine, or fails.
detect_platform() {
    os=$(uname -s)
    machine=$(uname -m)

    case "$os" in
    Linux) os=linux ;;
    Darwin) os=macos ;;
    MINGW* | MSYS* | CYGWIN*) die "this script does not support Windows; use the MSI from https://github.com/$repo/releases" ;;
    *) die "no prebuilt jpv for $machine-$os" ;;
    esac

    case "$machine" in
    x86_64 | amd64) arch=x86_64 ;;
    aarch64 | arm64) arch=aarch64 ;;
    *) die "no prebuilt jpv for $machine-$os" ;;
    esac

    # The platforms releases are built for.
    case "$arch-$os" in
    x86_64-linux | aarch64-macos) ;;
    *) die "no prebuilt jpv for $arch-$os; see https://github.com/$repo#building-and-installing" ;;
    esac

    printf '%s\n' "$arch-$os"
}

# Reads a URL to stdout, or to the file given as the second argument.
fetch() {
    if command -v curl >/dev/null 2>&1; then
        if [ $# -gt 1 ]; then
            curl -fsSL -o "$2" "$1"
        else
            curl -fsSL "$1"
        fi
    elif command -v wget >/dev/null 2>&1; then
        if [ $# -gt 1 ]; then
            wget -q -O "$2" "$1"
        else
            wget -q -O - "$1"
        fi
    else
        die "need curl or wget"
    fi
}

# Picks the download URL ending in -<platform>.tar.gz from release JSON on stdin.
pick_asset() {
    grep -o '"browser_download_url": *"[^"]*"' |
        sed 's/^[^:]*: *"//; s/"$//' |
        grep -e "-$1\\.tar\\.gz\$" |
        head -n 1
}

# Prints the archive URL for <platform> in the release at <url>, or nothing.
find_asset() {
    release=$(fetch "$2") || return 0
    printf '%s\n' "$release" | pick_asset "$1" || true
}

# main
command -v tar >/dev/null 2>&1 || die "need tar"

platform=$(detect_platform)
dir=${JPV_INSTALL_DIR:-$HOME/.local/bin}

if [ -n "${JPV_VERSION:-}" ]; then
    say "looking up release $JPV_VERSION"
    url=$(find_asset "$platform" "$api/tags/$JPV_VERSION")
    [ -n "$url" ] || die "no prebuilt jpv for $platform in release $JPV_VERSION"
else
    say "looking up the latest release"
    url=$(find_asset "$platform" "$api/latest")

    if [ -z "$url" ]; then
        say "the latest release has no $platform archive; trying nightly"
        url=$(find_asset "$platform" "$api/tags/nightly")
    fi

    [ -n "$url" ] || die "no prebuilt jpv for $platform; see https://github.com/$repo/releases"
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM

say "downloading $url"
fetch "$url" "$tmp/jpv.tar.gz" || die "download failed"
tar -xzf "$tmp/jpv.tar.gz" -C "$tmp" jpv || die "archive has no jpv binary"

mkdir -p "$dir"
mv "$tmp/jpv" "$dir/jpv"
chmod +x "$dir/jpv"
say "installed $dir/jpv"

case ":$PATH:" in
*":$dir:"*) ;;
*) say "warning: $dir is not on your PATH; add it to your shell profile" ;;
esac

say "next steps:"
say "  jpv build                  # download and build the dictionary (once)"
say "  jpv service --background   # run the dictionary service in the background"
say "OCR is not included in this build; use the Fedora RPM or build from source for it."
