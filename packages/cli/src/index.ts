import { cac, Command } from 'cac';
import colors from 'colors';
import { create } from './create/index.js';
import { COMMANDS } from './plugin/index.js';
import { resolveCore, log } from './utils.js';

const cli = cac();

cli
  .command(
    'start',
    'Compile the project in dev mode and serve it with farm dev server'
  )
  .action(async () => {
    const cwd = process.cwd();
    const { start } = await resolveCore(cwd);
    // TODO set config path
    start({
      configPath: cwd,
    });
  });

cli
  .command('build', 'Compile the project in production mode')
  .action(async () => {
    const cwd = process.cwd();
    const { build } = await resolveCore(cwd);
    // TODO set config path
    build({
      configPath: cwd,
    });
  });

cli.command('create [name]', 'Create a new project').action((name: string) => {
  create(name);
});

// 对未知命令监听
cli.on('command:*', function(obj: { args: string[] }){
  const availableCommands = cli.commands.map((cmd: Command) => cmd.name);
  console.log(colors.red(`未知的命令：${obj.args[0]}`));
  if(availableCommands.length > 0){
      console.log(colors.red(`可用命令：${availableCommands.join(',')}`));
  }
});

cli.command('').action(() => {
  cli.outputHelp();
});

const pluginCmd = cli.command(
  'plugin <command>',
  'Commands for manage plugins',
  {
    allowUnknownOptions: true,
  }
);
pluginCmd.action((command: keyof typeof COMMANDS, args: unknown) => {
  COMMANDS[command](args);
});

pluginCmd.cli.help();

cli.help();
try {
  cli.parse();
} catch (e) {
  // TODO error handling
  log('error',e.message);
  if(process.env.LOG_LEVEL === 'verbose'){
      console.log(e);
  }
}
