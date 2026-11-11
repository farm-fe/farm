// Experimental test targets only. This is not a publish/release inventory.
const targets = {
  "aarch64-apple-darwin": { abi: "darwin-arm64", extension: ".dylib" },
  "x86_64-unknown-linux-gnu": { abi: "linux-x64-gnu", extension: ".so" },
  "x86_64-pc-windows-msvc": { abi: "win32-x64-msvc", extension: ".dll" },
};

export function nativeTarget(target) {
  if (!Object.hasOwn(targets, target))
    throw new Error(`Unsupported experimental Federation target: ${target}`);
  return { target, ...targets[target] };
}

export function hostNativeTarget() {
  let suffix = "";
  if (process.platform === "win32") suffix = "-msvc";
  else if (process.platform === "linux")
    suffix = process.report?.getReport().header?.glibcVersionRuntime ? "-gnu" : "-musl";
  const abi = `${process.platform}-${process.arch}${suffix}`;
  const target = Object.keys(targets).find((key) => targets[key].abi === abi);
  if (!target)
    throw new Error(`Unsupported experimental Federation platform: ${abi}`);
  return nativeTarget(target);
}

export function nativeBuildOptions(args, env = process.env) {
  const options = new Map();
  const cargoArgs = [];
  const setOption = (name, value) => {
    if (!value || value.startsWith("-"))
      throw new Error(`Missing value for ${name}`);
    if (options.has(name)) throw new Error(`Repeated option: ${name}`);
    options.set(name, value);
  };
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    const name = arg.split("=", 1)[0];
    if (["--profile", "--target", "--abi"].includes(name)) {
      setOption(name, arg.includes("=") ? arg.slice(name.length + 1) : args[++i]);
    } else if (arg === "--release" || arg === "-r") {
      setOption("--profile", "release");
    } else if (name === "--message-format") {
      throw new Error("The Federation builder requires Cargo JSON messages");
    } else if (arg !== "--") {
      cargoArgs.push(arg);
    }
  }
  // Always pass a target so Cargo config cannot silently select another ABI.
  const target = options.get("--target") || env.CARGO_BUILD_TARGET;
  const native = target ? nativeTarget(target) : hostNativeTarget();
  const abi = options.get("--abi");
  if (abi && abi !== native.abi)
    throw new Error(`ABI ${abi} does not match target ${native.target} (${native.abi})`);
  return { ...native, profile: options.get("--profile") || "ci", cargoArgs };
}
