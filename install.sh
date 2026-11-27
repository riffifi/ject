#!/bin/sh
set -eu

usage() {
    cat <<'EOF'
Usage: sh install.sh [install|update] [latest|vVERSION]
       sh install.sh uninstall
       jectup update [latest|vVERSION]
       jectup uninstall

Ject installs to ~/.local/bin by default.
Set JECT_BIN_DIR to choose a different directory.
Set JECT_DATA_DIR to choose where the managed executable is stored.
EOF
}

fail() {
    printf 'Ject installer: %s\n' "$*" >&2
    exit 1
}

absolute() {
    case $1 in
        /*) printf '%s\n' "$1" ;;
        *) printf '%s/%s\n' "$PWD" "$1" ;;
    esac
}

command=${1:-install}
version=${2:-latest}
case $command in
    install|update) [ "$#" -le 2 ] || fail "expected at most one release tag" ;;
    uninstall) [ "$#" -eq 1 ] || fail "uninstall takes no release tag" ;;
    --help|-h) usage; exit 0 ;;
    *) usage >&2; exit 2 ;;
esac
case $version in
    latest) ;;
    v[0-9]*)
        case $version in *[!a-zA-Z0-9.+-]*) fail "invalid release tag: $version" ;; esac ;;
    *) fail "expected a release tag such as v0.11.0" ;;
esac

bin_dir=$(absolute "${JECT_BIN_DIR:-$HOME/.local/bin}")
data_dir=$(absolute "${JECT_DATA_DIR:-${XDG_DATA_HOME:-$HOME/.local/share}/ject}/distribution")
case $0 in
    */jectup)
        if [ -L "$0" ]; then
            bin_dir=$(absolute "$(dirname "$0")")
            data_dir=$(dirname "$(readlink "$0")")
        fi
        ;;
esac
link="$bin_dir/ject"
managed="$data_dir/ject"
manager_link="$bin_dir/jectup"
manager="$data_dir/jectup"

owned() {
    [ -L "$link" ] && [ "$(readlink "$link")" = "$managed" ]
}

owns_manager() {
    [ -L "$manager_link" ] && [ "$(readlink "$manager_link")" = "$manager" ]
}

if [ "$command" = uninstall ]; then
    owned || fail "no installer-managed Ject at $link"
    unlink "$link"
    if [ -e "$managed" ] || [ -L "$managed" ]; then
        unlink "$managed"
    fi
    if owns_manager; then
        unlink "$manager_link"
        if [ -e "$manager" ] || [ -L "$manager" ]; then unlink "$manager"; fi
    fi
    rmdir "$data_dir" 2>/dev/null || :
    printf 'Uninstalled Ject from %s\n' "$link"
    exit 0
fi

if [ -e "$link" ] || [ -L "$link" ]; then
    owned || fail "$link belongs to another installation. Set JECT_BIN_DIR to a different directory"
elif [ "$command" = update ]; then
    fail "no installer-managed Ject at $link. Run install first"
fi
if [ -e "$manager_link" ] || [ -L "$manager_link" ]; then
    owns_manager || fail "$manager_link belongs to another installation"
fi
if ! owned && { [ -e "$managed" ] || [ -L "$managed" ]; }; then
    fail "$managed exists without an installer-managed launcher"
fi

system=$(uname -s)
machine=$(uname -m)
case "$system:$machine" in
    Linux:x86_64) platform=x86_64-unknown-linux-gnu ;;
    Linux:aarch64|Linux:arm64) platform=aarch64-unknown-linux-gnu ;;
    Darwin:x86_64) platform=x86_64-apple-darwin ;;
    Darwin:arm64) platform=aarch64-apple-darwin ;;
    *) fail "no prebuilt release for $system $machine" ;;
esac

asset="ject-$platform"
if [ "$version" = latest ]; then
    release='https://github.com/riffifi/ject/releases/latest/download'
else
    release="https://github.com/riffifi/ject/releases/download/$version"
fi

mkdir -p "$data_dir" "$bin_dir"
scratch=$(mktemp -d "$data_dir/.install.XXXXXX")
trap 'rm -r "$scratch"' EXIT
trap 'exit 1' HUP INT TERM

download() {
    if [ -n "${JECT_RELEASE_DIR:-}" ]; then
        cp "$JECT_RELEASE_DIR/$1" "$scratch/$1"
    else
        curl -fsSL --proto '=https' --proto-redir '=https' --retry 2 \
            "$release/$1" -o "$scratch/$1"
    fi
}

download "$asset" || fail "could not download $asset for $version"
download "$asset.sha256" || fail "could not download the checksum"
expected=$(awk 'NR == 1 {print tolower($1)}' "$scratch/$asset.sha256")
case $expected in *[!0123456789abcdef]*|'') fail "invalid checksum file" ;; esac
[ "${#expected}" -eq 64 ] || fail "invalid checksum length"
if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum < "$scratch/$asset")
else
    actual=$(shasum -a 256 < "$scratch/$asset")
fi
[ "${actual%% *}" = "$expected" ] || fail "checksum mismatch for $asset"
chmod 755 "$scratch/$asset"
reported=$("$scratch/$asset" --version) || fail "downloaded Ject could not start"
case $reported in 'ject '[0-9]*) ;; *) fail "unexpected version output: $reported" ;; esac
if [ "$version" != latest ]; then
    [ "$reported" = "ject ${version#v}" ] || fail "wanted $version, downloaded $reported"
fi

[ -f "$0" ] || fail "save install.sh to a file before running it"
cp "$0" "$scratch/jectup"
chmod 755 "$scratch/jectup"
if ! owns_manager; then
    ln -s "$manager" "$manager_link" || fail "cannot create $manager_link"
fi
mv -f "$scratch/jectup" "$manager"

if owned; then
    mv -f "$scratch/$asset" "$managed"
else
    # Create the link without replacing a launcher another installer may have added.
    ln -s "$managed" "$link" || fail "cannot create $link"
    if ! mv -f "$scratch/$asset" "$managed"; then
        unlink "$link"
        fail "cannot place the executable at $managed"
    fi
fi

printf 'Installed %s at %s\n' "$reported" "$link"
if [ "$(command -v ject || :)" != "$link" ]; then
    printf 'Put %s first on PATH to use this installation.\n' "$bin_dir"
fi
