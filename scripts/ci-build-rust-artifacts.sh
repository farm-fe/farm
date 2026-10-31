#!/usr/bin/env sh
set -eu

node ./scripts/check-native-build-node.mjs

PLUGINS="react replace-dirname sass vue vue-jsx auto-import compress dsv dts icons image mdx modular-import react-components strip svgr url virtual wasm worker yaml tailwindcss"
PROFILE="${FARM_BUILD_PROFILE:-ci}"
ABI="${FARM_BUILD_ABI:?FARM_BUILD_ABI is required}"
TARGET="${FARM_BUILD_TARGET:-}"
ZIG="${FARM_BUILD_ZIG:-false}"

# Docs plugins declare five native targets; do not fabricate unsupported payloads.
case "$ABI" in
  win32-x64-msvc|linux-x64-gnu|linux-arm64-gnu|darwin-x64|darwin-arm64)
    PLUGINS="$PLUGINS ssr-css syntax-highlight docs docs-snapshot docs-seo docs-snippets"
    ;;
esac

BUILD_CORE=true
BUILD_CREATE=true
if [ -n "${FARM_RELEASE_NATIVE_OWNERS:-}" ]; then
  BUILD_CORE="$(node scripts/release-native-scope.mjs core "$PLUGINS")"
  BUILD_CREATE="$(node scripts/release-native-scope.mjs create "$PLUGINS")"
  PLUGINS="$(node scripts/release-native-scope.mjs plugins "$PLUGINS")"
fi

if [ -z "${CARGO_TARGET_DIR:-}" ]; then
  export CARGO_TARGET_DIR="$(pwd)/target"
fi

if [ "$PROFILE" = "ci" ]; then
  CORE_BUILD_SCRIPT="build:rs:ci"
  CREATE_FARM_BUILD_SCRIPT="build:ci"
  PLUGIN_PROFILE_ARGS="--profile ci"
else
  CORE_BUILD_SCRIPT="build:rs:publish"
  CREATE_FARM_BUILD_SCRIPT="build"
  PLUGIN_PROFILE_ARGS=""
fi

TARGET_ARGS=""
if [ -n "$TARGET" ]; then
  TARGET_ARGS="--target $TARGET"
fi

ZIG_ARGS=""
if [ "$ZIG" = "true" ]; then
  ZIG_ARGS="-x"
fi

case "$ABI" in
  linux-x64-gnu)
    unset CC_x86_64_unknown_linux_gnu || true
    unset CC || true
    export CARGO_INCREMENTAL=0
    if [ "$PROFILE" = "ci" ]; then
      # The live GNU reuse protocol requires a measured explicit linker.
      export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=gcc
    fi
    ;;
  linux-x64-musl)
    if [ -f /etc/alpine-release ]; then
      rm -f /etc/apk/repositories
      echo "https://dl-cdn.alpinelinux.org/alpine/v3.21/main" >> /etc/apk/repositories
      echo "https://dl-cdn.alpinelinux.org/alpine/v3.21/community" >> /etc/apk/repositories
      apk update
      apk add --upgrade clang-static llvm-dev
    fi
    export CARGO_INCREMENTAL=0
    if [ "$PROFILE" != "ci" ]; then
      export CARGO_PROFILE_RELEASE_LTO=false
    fi
    ;;
  win32-ia32-msvc|win32-arm64-msvc)
    cargo install cargo-xwin@0.18.6 --locked
    if [ "$PROFILE" != "ci" ]; then
      export CARGO_PROFILE_RELEASE_LTO=false
      if [ "$ABI" = "win32-arm64-msvc" ]; then
        export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=256
      fi
    fi
    ;;
esac

build_create_farm() {
  cd packages/create-farm
  npm run "$CREATE_FARM_BUILD_SCRIPT" -- $TARGET_ARGS $ZIG_ARGS
  cd ../..
}

CORE_EXTRA_ARGS=""
case "$ABI" in
  win32-ia32-msvc|win32-arm64-msvc)
    CORE_EXTRA_ARGS="--no-default-features"
    if [ "$PROFILE" != "ci" ]; then
      CORE_BUILD_SCRIPT="build:rs"
    fi
    ;;
esac

if [ "${FARM_NATIVE_PROOF_MODE:-off}" = "shadow" ]; then
  FARM_CAPTURE_CORE_SCRIPT="$CORE_BUILD_SCRIPT" \
  FARM_CAPTURE_CORE_ARGS="$TARGET_ARGS $ZIG_ARGS $CORE_EXTRA_ARGS" \
  FARM_CAPTURE_PLUGIN_ARGS="$TARGET_ARGS $ZIG_ARGS --abi $ABI $PLUGIN_PROFILE_ARGS" \
  FARM_CAPTURE_CREATE_SCRIPT="$CREATE_FARM_BUILD_SCRIPT" \
  FARM_CAPTURE_PLUGINS="$PLUGINS" \
    node scripts/ci-native-capture.mjs runtime || printf '%s\n' 'Shadow native context unavailable; reuse remains disabled'
fi

if [ "$BUILD_CORE" = true ]; then
  cd packages/core
  npm run "$CORE_BUILD_SCRIPT" -- $TARGET_ARGS $ZIG_ARGS $CORE_EXTRA_ARGS
  cd ../..
fi

if [ "$BUILD_CREATE" = true ]; then build_create_farm; fi

cd rust-plugins
for plugin in $PLUGINS; do
  cd "$plugin"
  npm run build -- $TARGET_ARGS $ZIG_ARGS --abi "$ABI" $PLUGIN_PROFILE_ARGS
  cd ..
done
