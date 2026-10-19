#!/bin/sh
here="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
sys="$(uname -s)"
case "$sys" in
MINGW* | MSYS* | CYGWIN*) rel="x86_64-pc-windows-msvc/gray-bookmark.exe" ;;
Darwin*)
    case "$(uname -m)" in
    arm64) rel="aarch64-apple-darwin/gray-bookmark" ;;
    *) rel="x86_64-apple-darwin/gray-bookmark" ;;
    esac ;;
*)
    case "$(uname -m)" in
    aarch64 | arm64) rel="aarch64-unknown-linux-musl/gray-bookmark" ;;
    *) rel="x86_64-unknown-linux-musl/gray-bookmark" ;;
    esac ;;
esac
if [ -x "$here/$rel" ]; then exec "$here/$rel" "$@"; fi
for cand in "$here/target/release/gray-bookmark.exe" "$here/target/release/gray-bookmark" "$here/target/debug/gray-bookmark.exe" "$here/target/debug/gray-bookmark"; do
    if [ -x "$cand" ]; then exec "$cand" "$@"; fi
done
echo "gray-bookmark binary not found in $here" >&2
exit 1
