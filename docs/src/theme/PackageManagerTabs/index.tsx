import { Tabs, Tab } from '../ManagerTabs';
import CodeBlock from "@theme/CodeBlock";
import { Npm } from './icons/Npm';
import { Yarn } from './icons/Yarn';
import { Pnpm } from './icons/Pnpm';
import { Bun } from './icons/Bun';

export interface PackageManagerTabProps {
  skip?: boolean;
  command:
  | string
  | {
    npm?: string;
    yarn?: string;
    pnpm?: string;
    bun?: string;
  };
  additionalTabs?: {
    tool: string;
    icon?: React.ReactNode;
  }[];
}

function normalizeCommand(command: string): string {
  if (!command?.includes('install')) {
    return command;
  }
  // If command include `install` and package name, replace `install` with `add`
  const pureCommand = command
    .split(' ')
    .filter(item => !item.startsWith('-') && !item.startsWith('--'))
    .join(' ');
  if (pureCommand === 'yarn install' || pureCommand === 'bun install') {
    return command;
  }

  return command.replace('install', 'add');
}


function replaceTool(command, tool) {
  const tools = ['npm', 'yarn', 'pnpm', 'bun'];
  let newCommand = command;

  tools.forEach(t => {
    const regex = new RegExp(`\\b${t}\\b`, 'g');
    newCommand = newCommand.replace(regex, tool);
  });

  return newCommand;
}

export function PackageManagerTabs({
  command,
  skip = true,
  additionalTabs = [],
}: PackageManagerTabProps) {
  let commandInfo: {
    npm?: string;
    yarn?: string;
    pnpm?: string;
    bun?: string;
    [key: string]: string;
  };

  // Init Icons
  const packageMangerToIcon = {
    npm: <Npm />,
    yarn: <Yarn />,
    pnpm: <Pnpm />,
    bun: <Bun />,
  };
  additionalTabs.forEach(tab => {
    packageMangerToIcon[tab.tool] = tab.icon;
  });

  // Init Command
  if (typeof command === 'string') {
    commandInfo = {
      npm: `npm ${command}`,
      yarn: `yarn ${command}`,
      pnpm: `pnpm ${command}`,
      bun: `bun ${command}`,
    };
    additionalTabs.forEach(tab => {
      if (skip) {
        commandInfo[tab.tool] = replaceTool(command, tab.tool);
      } else {
        commandInfo[tab.tool] = `${tab.tool} ${command}`;
      }
    });
  } else {
    commandInfo = command;
  }

  // Normalize yarn/bun command
  commandInfo.yarn && (commandInfo.yarn = normalizeCommand(commandInfo.yarn));
  commandInfo.bun && (commandInfo.bun = normalizeCommand(commandInfo.bun));
  if (skip) {
    const tools = ['npm', 'yarn', 'pnpm', 'bun'];
    tools.forEach(tool => {
      commandInfo[tool] = replaceTool(command, tool);
    });
  }
  return (
    <Tabs
      groupId="package.manager"
      values={Object.entries(commandInfo).map(([key]) => (
        <div
          key={key}
          style={{
            display: 'flex',
            alignItems: 'center',
            fontSize: 15,
          }}
        >
          {packageMangerToIcon[key]}
          <span style={{ marginLeft: 6, marginBottom: 2 }}>{key}</span>
        </div>
      ))}
    >
      {Object.entries(commandInfo).map(([key, value]) => (
        <Tab key={key}>
          <CodeBlock className='my-2'>{value}</CodeBlock>
        </Tab>
      ))}
    </Tabs>
  );
}
