// cspell:words RUSTFLAGS
import crypto from 'node:crypto';
const hash=value=>crypto.createHash('sha256').update(JSON.stringify(value)).digest('hex');
const fields=['RUST_LOG','RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_INCREMENTAL','CARGO_PROFILE_RELEASE_LTO','CARGO_PROFILE_RELEASE_CODEGEN_UNITS','FARM_BUILD_ABI','FARM_BUILD_TARGET','FARM_BUILD_ZIG','FARM_CAPTURE_CORE_SCRIPT','FARM_CAPTURE_CORE_ARGS','FARM_CAPTURE_PLUGIN_ARGS','FARM_CAPTURE_CREATE_SCRIPT'];
export function formalFlagsHash(env) { return hash(Object.fromEntries(fields.map(key=>[key,env[key]??null]))); }
