#!/bin/sh
set -eu

[ "$#" -eq 1 ] || { echo "Usage: sh scripts/test-install.sh JECT_BINARY" >&2; exit 2; }
installer=$(CDPATH= cd "$(dirname "$0")/.." && pwd)/install.sh
asset=$(basename "$1")
source=$1
scratch=$(mktemp -d)
trap 'rm -r "$scratch"' EXIT
export JECT_BIN_DIR="$scratch/my bin"
export JECT_DATA_DIR="$scratch/data"
export JECT_RELEASE_DIR="$scratch/assets"
mkdir -p "$JECT_RELEASE_DIR"
chmod +x "$source"
cp "$source" "$JECT_RELEASE_DIR/$asset"
expected=$("$source" --version)
tag="v${expected#ject }"

hash() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum < "$1" | awk '{ print $1 }'
    else
        shasum -a 256 < "$1" | awk '{ print $1 }'
    fi
}

digest=$(hash "$source")
printf '%s\n' "$digest" > "$JECT_RELEASE_DIR/$asset.sha256"
sh "$installer" install "$tag"
PATH="$JECT_BIN_DIR:$PATH"
export PATH
[ "$(command -v ject)" = "$JECT_BIN_DIR/ject" ]
[ "$(ject --version)" = "$expected" ]
mkdir "$scratch/work"
cd "$scratch/work"
ject init
ject check
ject run > "$scratch/run.out"
test -s "$scratch/run.out"

printf '%064d\n' 0 > "$JECT_RELEASE_DIR/$asset.sha256"
if sh "$installer" update; then
    echo "installer accepted an invalid checksum" >&2
    exit 1
fi
[ "$(ject --version)" = "$expected" ]
printf '%s\n' "$digest" > "$JECT_RELEASE_DIR/$asset.sha256"
if sh "$installer" update v999.999.999; then
    echo "installer accepted an incorrect release tag" >&2
    exit 1
fi
printf '#!/bin/sh\nprintf "ject 999.0.0\\n"\n' > "$JECT_RELEASE_DIR/$asset"
hash "$JECT_RELEASE_DIR/$asset" > "$JECT_RELEASE_DIR/$asset.sha256"
env -u JECT_BIN_DIR -u JECT_DATA_DIR jectup update v999.0.0
[ "$(ject --version)" = 'ject 999.0.0' ]
env -u JECT_BIN_DIR -u JECT_DATA_DIR jectup uninstall
[ ! -e "$JECT_BIN_DIR/ject" ] && [ ! -L "$JECT_BIN_DIR/ject" ]
[ ! -e "$JECT_BIN_DIR/jectup" ] && [ ! -L "$JECT_BIN_DIR/jectup" ]
printf 'foreign executable\n' > "$JECT_BIN_DIR/ject"
if sh "$installer" install; then
    echo "installer replaced an unmanaged file" >&2
    exit 1
fi
grep -Fx 'foreign executable' "$JECT_BIN_DIR/ject"
echo "Install, update, failed update, and uninstall passed"
