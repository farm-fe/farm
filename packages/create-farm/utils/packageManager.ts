import { execSync } from 'child_process';

// TODO use npx create-farm to create a new farm to use manager

export function shouldUsePnpm(): boolean {
  try {
    const userAgent = process.env.npm_config_user_agent;
    if (userAgent && userAgent.startsWith('pnpm')) {
      return true;
    }
    execSync('pnpm --version', { stdio: 'ignore' });
    return true;
  } catch (e) {
    return false;
  }
}

export function shouldUseYarn(): boolean {
  try {
    const userAgent = process.env.npm_config_user_agent;
    if (userAgent && userAgent.startsWith('yarn')) {
      return true;
    }
    execSync('yarnpkg --version', { stdio: 'ignore' });
    return true;
  } catch (e) {
    return false;
  }
}
