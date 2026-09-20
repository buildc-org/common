#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

if [ "${1:-}" = "--fix" ]; then
    cargo fmt
    echo "lint: formatted the source"
    exit
fi

deps=$(cargo tree -e normal --depth 1 --prefix none | awk 'NR > 1 {print $1}' | sort | tr '\n' ' ')
if [ "$deps" != "libc ratatui " ]; then
    echo "lint: the dependency floor moved, it is now: $deps" >&2
    exit 1
fi

cargo fmt --check || {
    echo "lint: unformatted, run ./lint.sh --fix" >&2
    exit 1
}
echo "lint: the source is clean"

cargo test --quiet
echo "lint: the components do what they did"
