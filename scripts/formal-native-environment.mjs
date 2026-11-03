// cspell:words RANLIB READELF
// cspell:words IPHONEOS TVOS WATCHOS XROS SDKROOT CXXFLAGS CPPFLAGS LDFLAGS
// cspell:words RUSTFLAGS
import crypto from 'node:crypto';
const hash=value=>crypto.createHash('sha256').update(JSON.stringify(value)).digest('hex');
// Explicit effective inputs inherited by the reviewed GNU image. Values are
// fingerprinted, never silently allowed as unmeasured environment.
export const gnuCompilerEnvironmentFields=['CC','CXX','AR','CC_x86_64_unknown_linux_gnu','CXX_x86_64_unknown_linux_gnu','AR_x86_64_unknown_linux_gnu','CARGO_BUILD_TARGET','CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER','RUST_TARGET','TARGET_CC','TARGET_CXX','TARGET_AR','TARGET_RANLIB','TARGET_READELF','TARGET_SYSROOT','TARGET_C_INCLUDE_PATH','TARGET_CMAKE_TOOLCHAIN_FILE_PATH','X86_64_UNKNOWN_LINUX_GNU_OPENSSL_DIR','C_INCLUDE_PATH'];
export const compilerEnvironmentFields=[...gnuCompilerEnvironmentFields,'MACOSX_DEPLOYMENT_TARGET','IPHONEOS_DEPLOYMENT_TARGET','TVOS_DEPLOYMENT_TARGET','WATCHOS_DEPLOYMENT_TARGET','XROS_DEPLOYMENT_TARGET','SDKROOT','CFLAGS','CXXFLAGS','CPPFLAGS','LDFLAGS','CARGO_BUILD_JOBS'];
const fields=[...compilerEnvironmentFields,'RUST_LOG','RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_INCREMENTAL','CARGO_PROFILE_RELEASE_STRIP', 'CARGO_PROFILE_RELEASE_LTO','CARGO_PROFILE_RELEASE_CODEGEN_UNITS','FARM_BUILD_ABI','FARM_BUILD_TARGET','FARM_BUILD_ZIG','FARM_CAPTURE_CORE_SCRIPT','FARM_CAPTURE_CORE_ARGS','FARM_CAPTURE_PLUGIN_ARGS','FARM_CAPTURE_CREATE_SCRIPT'];
export function formalFlagsHash(env) { return hash(Object.fromEntries(fields.map(key=>[key,env[key]??null]))); }
