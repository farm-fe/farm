#!/usr/bin/env node

import { execSync } from 'child_process';
import { resolve } from 'path';

const CWD = process.cwd();

console.log('Building Farm core package and its dependencies...');

try {
  // Build utils package first
  console.log('Building utils package...');
  execSync('npx tsc -p packages/utils/tsconfig.json', { 
    cwd: CWD, 
    stdio: 'inherit' 
  });

  // Build runtime packages
  console.log('Building runtime packages...');
  execSync('pnpm --filter @farmfe/runtime run build', { 
    cwd: CWD, 
    stdio: 'inherit' 
  });
  
  execSync('pnpm --filter @farmfe/runtime-plugin-hmr run build', { 
    cwd: CWD, 
    stdio: 'inherit' 
  });
  
  execSync('pnpm --filter @farmfe/runtime-plugin-import-meta run build', { 
    cwd: CWD, 
    stdio: 'inherit' 
  });

  // Finally build core package
  console.log('Building core package...');
  execSync('pnpm --filter farm run build', { 
    cwd: CWD, 
    stdio: 'inherit' 
  });

  console.log('✅ Farm core package built successfully!');
} catch (error) {
  console.error('❌ Build failed:', error.message);
  process.exit(1);
}
