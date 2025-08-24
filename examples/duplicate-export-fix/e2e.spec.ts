import { expect, test } from 'vitest';
import { readFileSync } from 'fs';
import { join } from 'path';
import { startProjectAndTest } from '../../e2e/vitestSetup.js';
import { describe } from 'node:test';

describe('Duplicate Export Fix E2E Tests', () => {
  test('should not generate duplicate variable declarations in ESM output', async () => {
    // Build the project first
    const { execSync } = await import('child_process');
    const examplePath = './examples/duplicate-export-fix';
    
    // Run build
    execSync('pnpm build', { 
      cwd: examplePath,
      stdio: 'inherit' 
    });
    
    // Check the generated files for duplicate variable declarations
    const complexOutput = readFileSync(join(examplePath, 'dist/complex.js'), 'utf-8');
    const namespaceOutput = readFileSync(join(examplePath, 'dist/namespace.js'), 'utf-8');
    
    // Count occurrences of variable declarations
    const countVarDeclarations = (content: string, varName: string) => {
      const regex = new RegExp(`var ${varName}=`, 'g');
      return (content.match(regex) || []).length;
    };
    
    // Test complex.js - 'shared' should only be declared once
    const sharedCount = countVarDeclarations(complexOutput, 'shared');
    expect(sharedCount).toBe(1);
    
    // Test namespace.js - 'formatDate' and 'formatTime' should only be declared once
    const formatDateCount = countVarDeclarations(namespaceOutput, 'formatDate');
    const formatTimeCount = countVarDeclarations(namespaceOutput, 'formatTime');
    expect(formatDateCount).toBe(1);
    expect(formatTimeCount).toBe(1);
    
    // Check that exports still exist (without var declarations)
    expect(complexOutput).toContain('export { shared }');
    expect(namespaceOutput).toContain('export { formatDate }');
    expect(namespaceOutput).toContain('export { formatTime }');
  });
  
  test('should render correctly in browser', async () => {
    await startProjectAndTest(
      './examples/duplicate-export-fix',
      async (page) => {
        // Wait for the test results div
        await page.waitForSelector('#test-results', { timeout: 10000 });
        
        // Check that all exports are working
        const resultsText = await page.$eval('#test-results', el => el.textContent);
        expect(resultsText).toContain('All exports are working correctly!');
        
        // Verify no JavaScript errors occurred
        const consoleErrors: string[] = [];
        page.on('console', msg => {
          if (msg.type() === 'error') {
            consoleErrors.push(msg.text());
          }
        });
        
        // Wait a bit to catch any async errors
        await page.waitForTimeout(1000);
        
        // No console errors should occur (especially no duplicate declaration errors)
        expect(consoleErrors).toHaveLength(0);
      }
    );
  });
});