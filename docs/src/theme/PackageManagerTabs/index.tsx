import { Tabs, Tab } from '../ManagerTabs';
import React from 'react';
import { Npm } from './icons/Npm';
import { Yarn } from './icons/Yarn';
import { Pnpm } from './icons/Pnpm';
import { Bun } from './icons/Bun';
import { codeToHtml } from 'shiki';
import { useEffect, useState } from 'react';
import useIsBrowser from '@docusaurus/useIsBrowser';
import { useColorMode } from '@docusaurus/theme-common';
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
    [key: string]: string | undefined;
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
  const [highlightedCode, setHighlightedCode] = useState({});
  const { colorMode } = useColorMode();
  const isBrowser = useIsBrowser();
  useEffect(() => {
    async function highlightCode() {
      if (isBrowser) {
        const { codeToHtml } = await import('shiki');
        const highlighted = {};
        for (const [key, value] of Object.entries(commandInfo)) {
          highlighted[key] = await codeToHtml(value as string, {
            lang: 'bash',
            theme: colorMode === 'dark' ? 'vitesse-dark' : 'vitesse-light',
          });
        }
        setHighlightedCode(highlighted);
      }
    }
    highlightCode();
  }, [command, colorMode, isBrowser]);

  return (
    <div className="border-solid my-4 rounded-md border-package">
      <Tabs
        groupId="package.manager"
        values={Object.entries(commandInfo).map(([key]) => (
          <div
            key={key}
            className="package-manager-tab"
            style={{
              display: 'flex',
              alignItems: 'center',
              padding: '2px 4px',
              borderRadius: '4px',
              transition: 'background-color 0.3s ease',
            }}
          >
            {packageMangerToIcon[key]}
            <span className="package-manager-name ">{key}</span>
          </div>
        ))}
      >
        {Object.entries(commandInfo).map(([key, value]) => (
          <Tab key={key}>
            <div className='codeBlock-tab shadow-md'
              dangerouslySetInnerHTML={{ __html: highlightedCode[key] || '' }}
            ></div>
          </Tab>
        ))}
      </Tabs>
    </div>
  );
}
