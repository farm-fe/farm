import envinfo from 'envinfo';

export interface EnvInfo {
  version: string;
  path: string;
}

export interface PackageInfo {
  installed: string;
  wanted: string;
}

export interface FarmEnvInfo {
  System: {
    OS: string;
    CPU: string;
    Memory: string;
    Shell: EnvInfo;
  };
  Binaries: {
    Node?: EnvInfo;
    Yarn?: EnvInfo;
    npm?: EnvInfo;
    pnpm?: EnvInfo;
  };
  Browsers: {
    Chrome?: EnvInfo;
    Firefox?: EnvInfo;
    Safari?: EnvInfo;
  };
  npmPackages: {
    farm: PackageInfo | string;
  };
}

export function getFarmEnvInfo() {
  return envinfo.run(
    {
      System: ['OS', 'CPU', 'Memory', 'Shell'],
      Binaries: ['Node', 'Yarn', 'npm', 'pnpm'],
      Browsers: ['Chrome', 'Firefox', 'Safari'],
      npmPackages: ['farm', 'react']
    },
    { json: true, showNotFound: true }
  );
}
