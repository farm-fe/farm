#!/usr/bin/env sh
# cspell:ignore libstdc SHASUMS mktemp
# Source this inside the existing GNU/musl Rust image so child builds inherit PATH.
set -eu

if [ "$(uname -s)" != Linux ] || [ "$(uname -m)" != x86_64 ]; then
  echo 'Native container Node bootstrap requires Linux x64.' >&2
  return 1
fi

farm_native_node_version=22.20.0
case "${FARM_BUILD_ABI:?FARM_BUILD_ABI is required}" in
  linux-x64-gnu)
    farm_native_node_variant=linux-x64
    farm_native_node_server=https://nodejs.org/dist/v22.20.0
    farm_native_node_sha=00bbd05e306ea68b6e13e17360d0e2f680b493ef95f2fea1c4296ff7437530bc
    ;;
  linux-x64-musl)
    # Node.js Build Working Group's musl archive, also used by docker-node.
    farm_native_node_variant=linux-x64-musl
    farm_native_node_server=https://unofficial-builds.nodejs.org/download/release/v22.20.0
    farm_native_node_sha=119817f0a2cb86e9b8f058a9fc0a9fcb1b56a8a6da7c6ebef64398328ee52d82
    apk add --no-cache curl xz libstdc++
    ;;
  *)
    echo "Unsupported native container ABI: $FARM_BUILD_ABI" >&2
    return 1
    ;;
esac

# Both pins were checked against each publisher's v22.20.0/SHASUMS256.txt.
# Leave this job-owned temporary runtime to the disposable container's lifecycle.
farm_native_node_dir=$(mktemp -d "${TMPDIR:-/tmp}/farm-native-node.XXXXXXXX")
farm_native_node_archive="node-v$farm_native_node_version-$farm_native_node_variant.tar.xz"
curl --fail --location --silent --show-error \
  --output "$farm_native_node_dir/$farm_native_node_archive" \
  "$farm_native_node_server/$farm_native_node_archive"
(
  cd "$farm_native_node_dir"
  printf '%s  %s\n' "$farm_native_node_sha" "$farm_native_node_archive" | sha256sum -c -
)
mkdir "$farm_native_node_dir/runtime"
tar -xJf "$farm_native_node_dir/$farm_native_node_archive" \
  -C "$farm_native_node_dir/runtime" --strip-components=1
export PATH="$farm_native_node_dir/runtime/bin:$PATH"
hash -r 2>/dev/null || true
test "$(command -v node)" = "$farm_native_node_dir/runtime/bin/node"
test "$(node -p 'process.version')" = "v$farm_native_node_version"
node --version
node -p 'JSON.stringify({execPath:process.execPath,node:process.version,platform:process.platform,arch:process.arch,glibc:process.report.getReport().header.glibcVersionRuntime})'
node ./scripts/check-native-build-node.mjs
