#!/usr/bin/env bash
# cspell:words userconfig globalconfig pathlib realpath printenv
set -euo pipefail
repo=$1
sha=$2
out=$3
base=$4
fixture=$(realpath "$5")
node=$(realpath "$6")
npm=$(realpath "$7")
export HTTP_PROXY=$(printenv HTTP_PROXY || true) HTTPS_PROXY=$(printenv HTTPS_PROXY || true) NODE_EXTRA_CA_CERTS=$(printenv NODE_EXTRA_CA_CERTS || true)
test "$#" = 7
test ! -e "$base"
test "$("$node" --version)" = v22.22.2
test "$(git -C "$repo" rev-parse "$sha^{commit}")" = "$sha"
test "$(printf '%s' "$sha" | wc -c)" = 40
mkdir -p "$base/home"
: >"$base/home/user.npmrc"
: >"$base/home/global.npmrc"
export PATH="$(dirname "$node"):/usr/bin:/bin"
# Allow the runner's credential-free proxy and CA only; omit ambient npm auth.
for proxy in "$HTTP_PROXY" "$HTTPS_PROXY"; do
  "$node" -e 'if(process.argv[1]) { const u=new URL(process.argv[1]); if(u.username||u.password||!["http:","https:"].includes(u.protocol))process.exit(1) }' "$proxy"
done
clean() { env -i PATH="$PATH" HOME="$base/home" HTTPS_PROXY="$HTTPS_PROXY" HTTP_PROXY="$HTTP_PROXY" NODE_EXTRA_CA_CERTS="$NODE_EXTRA_CA_CERTS" npm_config_userconfig="$base/home/user.npmrc" npm_config_globalconfig="$base/home/global.npmrc" npm_config_fetch_retries=0 "$@"; }
test "$(clean "$node" "$npm" --version)" = 11.19.1
git -C "$repo" archive "$sha" -- examples/docs-minimal examples/docs-features | tar xf - -C "$base"
for example in docs-minimal docs-features; do
  cd "$base/examples/$example"
  OUT="$out" FIXTURE="$fixture/package.json" "$node" --input-type=module <<'JS'
import fs from 'node:fs';
const deps=JSON.parse(fs.readFileSync(process.env.FIXTURE)).dependencies;
fs.writeFileSync('package.json', JSON.stringify({name:'clean-docs-pair-consumer',version:'0.0.0',type:'module',private:true,dependencies:{...deps,
'@farmfe/docs':'file:'+process.env.OUT+'/farmfe-docs-2.0.0.tgz',
'@farmfe/docs-theme-default':'file:'+process.env.OUT+'/farmfe-docs-theme-default-2.0.0.tgz'}})+'\n');
JS
  clean "$node" "$npm" install --package-lock-only --ignore-scripts --no-audit --no-fund --registry=https://registry.npmjs.org/
  clean "$node" "$npm" ci --ignore-scripts --no-audit --no-fund --registry=https://registry.npmjs.org/
  OUT="$out" SOURCE_SHA="$sha" "$node" --input-type=module <<'JS'
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
const receipt=JSON.parse(fs.readFileSync(path.join(process.env.OUT,'frozen-pair.json')));
const lock=JSON.parse(fs.readFileSync('package-lock.json'));
assert.equal(receipt.sourceSHA,process.env.SOURCE_SHA);
for (const p of receipt.packages) {
 const d='node_modules/'+p.name, l=lock.packages[d];
 assert.equal(l.version,p.version);assert.equal(l.integrity,p.integrity);
 assert.equal(fs.lstatSync(d).isSymbolicLink(),false);
 assert.equal(JSON.parse(fs.readFileSync(d+'/package.json')).version,p.version);
}
assert.equal(JSON.parse(fs.readFileSync('node_modules/@farmfe/docs/package.json')).dependencies['@farmfe/docs-theme-default'],'2.0.0');
assert.equal(JSON.parse(fs.readFileSync('node_modules/@farmfe/docs-theme-default/package.json')).peerDependencies['@farmfe/docs'],'2.0.0');
console.log('Sealed tarball pair installed and locked exactly');
JS
  clean "$node" node_modules/.bin/farm build
  count=$(find build -name '*.html' -type f | wc -l)
  case $example in
    docs-minimal) test "$count" = 3;;
    docs-features) test "$count" = 10;;
  esac
  printf '%s: %s emitted pages\n' "$example" "$count"
done

python3 - "$base" <<'PY2'
import sys
from pathlib import Path
root = Path(sys.argv[1]) / 'examples'
for relative, expected in {
    'docs-minimal/build/getting-started/index.html': ('lang="en"', 'Install Farm Docs', 'id="your-first-page"'),
    'docs-features/build/docs/index.html': ('lang="en"', 'Current documentation', '/guide/zh/docs/'),
    'docs-features/build/zh/docs/index.html': ('lang="zh"', '当前版本文档', '/guide/docs/'),
    'docs-features/build/zh/docs/modern/index.html': ('lang="zh"', '当前版本新增功能', 'id="stable-modern"'),
    'docs-features/build/zh/docs/untranslated/index.html': ('lang="zh"', 'Content from English', '/guide/docs/untranslated/'),
}.items():
    text = (root / relative).read_text()
    assert all(part in text for part in expected), relative
print('Exact English, Chinese, fallback and anchor SSR assertions passed.')
PY2
