#!/usr/bin/env sh
set -eu

PLUGINS="react replace-dirname sass auto-import compress dsv dts icons image mdx modular-import react-components strip svgr url virtual wasm worker yaml tailwindcss"
PROFILE="${FARM_BUILD_PROFILE:-ci}"
ABI="${FARM_BUILD_ABI:?FARM_BUILD_ABI is required}"
TARGET="${FARM_BUILD_TARGET:-}"
ZIG="${FARM_BUILD_ZIG:-false}"

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
    if command -v apt-get >/dev/null 2>&1; then
      apt-get update
      apt-get install -y protobuf-compiler --fix-missing
    fi
    ;;
  linux-x64-musl)
    if [ -f /etc/alpine-release ]; then
      rm -f /etc/apk/repositories
      echo "https://dl-cdn.alpinelinux.org/alpine/v3.21/main" >> /etc/apk/repositories
      echo "https://dl-cdn.alpinelinux.org/alpine/v3.21/community" >> /etc/apk/repositories
      apk update
      apk add --upgrade clang-static llvm-dev protobuf
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

cd packages/core
npm run "$CORE_BUILD_SCRIPT" -- $TARGET_ARGS $ZIG_ARGS $CORE_EXTRA_ARGS
cd ../..

build_create_farm

cd rust-plugins
for plugin in $PLUGINS; do
  cd "$plugin"
  npm run build -- $TARGET_ARGS $ZIG_ARGS --abi "$ABI" $PLUGIN_PROFILE_ARGS
  cd ..
done
