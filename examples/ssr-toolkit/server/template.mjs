import fs from 'node:fs/promises';
import path from 'node:path';

function escapeHtml(raw) {
  return String(raw)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

export function renderEjsLike(template, locals) {
  return template.replace(/<%([=-])\s*([a-zA-Z_$][a-zA-Z0-9_$]*)\s*%>/g, (_all, mode, key) => {
    const value = locals[key] ?? '';

    if (mode === '=') {
      return escapeHtml(value);
    }

    return String(value);
  });
}

export function injectAppHtmlIntoBuiltTemplate(template, appHtml) {
  const rootContainerPattern = /<div\s+id=(?:"root"|'root'|root)[^>]*>[\s\S]*?<\/div>/;

  if (!rootContainerPattern.test(template)) {
    throw new Error('[ssr-toolkit] failed to locate root container in built html template.');
  }

  return template.replace(rootContainerPattern, `<div id="root">${appHtml}</div>`);
}

export function createTemplateConfig(params) {
  const { command, templateMode } = params;

  if (command === 'preview' && templateMode === 'ejs') {
    throw new Error(
      '[ssr-toolkit] preview mode does not support SSR_TEMPLATE_MODE=ejs yet. Use default html template for host:preview.'
    );
  }

  if (templateMode === 'ejs') {
    return {
      file: './index.ejs',
      async load({ root }) {
        return fs.readFile(path.join(root, 'index.ejs'), 'utf-8');
      },
      async transform({ template, appHtml }) {
        return renderEjsLike(template, {
          appHtml,
          templateMode,
          pageTitle: 'Farm SSR Toolkit Example (ejs)'
        });
      }
    };
  }

  if (command === 'preview') {
    return {
      file: './dist/client/index.html',
      async transform({ template, appHtml }) {
        return injectAppHtmlIntoBuiltTemplate(template, appHtml);
      }
    };
  }

  return {
    resource: 'index.html',
    placeholder: '<!--app-html-->'
  };
}

export function createSsrRenderConfig(params) {
  const { command, templateMode } = params;

  return {
    ...(command === 'dev' ? { entry: '/src/entry-server.ts' } : {}),
    template: createTemplateConfig({
      command,
      templateMode
    })
  };
}

