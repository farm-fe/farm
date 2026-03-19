const SSR_SUBCOMMANDS = new Set(['dev', 'build', 'preview']);

export function normalizeCliArgv(argv: string[]) {
  if (argv.length < 4) {
    return argv;
  }

  if (argv[2] !== 'ssr') {
    return argv;
  }

  const subcommand = argv[3];
  if (!SSR_SUBCOMMANDS.has(subcommand)) {
    return argv;
  }

  const normalized = [...argv];
  normalized.splice(2, 2, `ssr ${subcommand}`);
  return normalized;
}
