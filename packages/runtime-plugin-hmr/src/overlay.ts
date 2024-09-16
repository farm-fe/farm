const base = '/';

// set :host styles to make playwright detect the element as visible
const template = /*html*/ `
<style>
:host {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 99999;
  --monospace: 'SFMono-Regular', Consolas,
  'Liberation Mono', Menlo, Courier, monospace;
  --red: #ff5555;
  --brand-color: #9f1a8f;
  --window-background: #ffffff;
  --window-color: #d8d8d8;
  --brand-color-o: rgba(248, 44, 224, 0.3);
}

.backdrop {
  position: fixed;
  z-index: 99999;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow-y: scroll;
  margin: 0;
  background: rgba(0, 0, 0, 0.56);
}

.window {
  font-family: var(--monospace);
  line-height: 1.5;
  width: 800px;
  color: var(--window-color);
  margin: 60px auto;
  position: relative;
  background: var(--window-background);
  border-radius: 6px 6px 8px 8px;
  box-shadow: 0 20px 34px rgba(0,0,0,0.40), 0 18px 16px rgba(0,0,0,0.30);
  overflow: hidden;
  border-top: 6px solid var(--red);
  direction: ltr;
  text-align: left;
}

pre {
  font-family: var(--monospace);
  font-size: 14px;
  margin-top: 0;
  margin-bottom: 1em;
  overflow-x: scroll;
  scrollbar-width: none;
}

pre::-webkit-scrollbar {
  display: none;
}

pre.frame::-webkit-scrollbar {
  display: block;
  height: 5px;
}

pre.frame::-webkit-scrollbar-thumb {
  background: #999;
  border-radius: 5px;
}

pre.frame {
  scrollbar-width: thin;
}

.message {
  max-height: 400px;
  padding: 25px 30px;
  line-height: 1.3;
  font-weight: 600;
  white-space: pre-wrap;
}

.message-body {
  color: var(--red);
}

code {
  font-size: 13px;
  font-family: var(--monospace);
  color: var(--yellow);
}

.footer {
  font-family: var(--monospace);
  background: rgba(0, 0, 0, 0.16);
  color: #000;
  padding: 10px 20px;
  border-radius: 0 0 6px 6px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.tip {
  font-size: 13px;
  line-height: 1.8;
}

kbd {
  line-height: 1.5;
  font-family: ui-monospace, Menlo, Monaco, Consolas, 'Liberation Mono',
    'Courier New', monospace;
  font-size: 0.75rem;
  font-weight: 700;
  background-color: rgb(38, 40, 44);
  color: rgb(166, 167, 171);
  padding: 0.15rem 0.3rem;
  border-radius: 0.25rem;
  border-width: 0.0625rem 0.0625rem 0.1875rem;
  border-style: solid;
  border-color: rgb(54, 57, 64);
  border-image: initial;
}

kbd {
  line-height: 1.5;
  font-family: ui-monospace, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  font-size: 0.75rem;
  font-weight: 700;
  background-color: rgb(38, 40, 44);
  color: rgb(166, 167, 171);
  padding: 0.15rem 0.3rem;
  border-radius: 0.25rem;
  border-width: 0.0625rem 0.0625rem 0.1875rem;
  border-style: solid;
  border-color: rgb(54, 57, 64);
  border-image: initial;
}
</style>
<div class="backdrop" part="backdrop">
  <div class="window" part="window">
    <pre class="message" part="message"><span class="plugin" part="plugin"></span><span class="message-body" part="message-body"></span></pre>
    <div class="footer">
      <div class="tip" part="tip">
        Click outside, press <kbd>Esc</kbd> key, or fix the code to dismiss.<br />
      </div>
    </div>
  </div>
</div>
`;

const fileRE = /(?:[a-zA-Z]:\\|\/).*?:\d+:\d+/g;
const codeframeRE = /^(?:>?\s*\d+\s+\|.*|\s+\|\s*\^.*)\r?\n/gm;

// Allow `ErrorOverlay` to extend `HTMLElement` even in environments where
// `HTMLElement` was not originally defined.
const { HTMLElement = class {} as typeof globalThis.HTMLElement } = globalThis;
export class ErrorOverlay extends HTMLElement {
  root: ShadowRoot;
  closeOnEsc: (e: KeyboardEvent) => void;
  // TODO Optimize hmr boundaries and return values to match vite hmr mode socket return values
  constructor(err: any, links = true) {
    super();
    this.root = this.attachShadow({ mode: 'open' });
    this.root.innerHTML = template;

    codeframeRE.lastIndex = 0;
    const hasFrame = err.frame && codeframeRE.test(err.frame);
    const message = hasFrame
      ? err.message.replace(codeframeRE, '')
      : err.message;
    if (err.plugin) {
      this.text('.plugin', `[plugin:${err.plugin}] `);
    }
    this.text('.message-body', message);

    const [file] = (err.loc?.file || err.id || 'unknown file').split(`?`);
    if (err.loc) {
      this.text('.file', `${file}:${err.loc.line}:${err.loc.column}`, links);
    } else if (err.id) {
      this.text('.file', file);
    }

    if (hasFrame) {
      this.text('.frame', err.frame?.trim());
    }
    this.text('.stack', err.stack, links);

    this.root.querySelector('.window')?.addEventListener('click', (e) => {
      e.stopPropagation();
    });

    this.addEventListener('click', () => {
      this.close();
    });

    this.closeOnEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape' || e.code === 'Escape') {
        this.close();
      }
    };

    document.addEventListener('keydown', this.closeOnEsc);
  }

  text(selector: string, text: string, linkFiles = false): void {
    const el = this.root.querySelector(selector)!;
    if (!linkFiles) {
      el.textContent = text;
    } else {
      let curIndex = 0;
      let match: RegExpExecArray | null;
      fileRE.lastIndex = 0;
      while ((match = fileRE.exec(text))) {
        const { 0: file, index } = match;
        if (index != null) {
          const frag = text.slice(curIndex, index);
          el.appendChild(document.createTextNode(frag));
          const link = document.createElement('a');
          link.textContent = file;
          link.className = 'file-link';
          link.onclick = () => {
            fetch(
              new URL(
                `${base}__open-in-editor?file=${encodeURIComponent(file)}`,
                // import.meta.url
                window.location.href
              )
            );
          };
          el.appendChild(link);
          curIndex += frag.length + file.length;
        }
      }
    }
  }
  close(): void {
    this.parentNode?.removeChild(this);
    document.removeEventListener('keydown', this.closeOnEsc);
  }
}

export const overlayId = 'vite-error-overlay';
const { customElements } = globalThis; // Ensure `customElements` is defined before the next line.
if (customElements && !customElements.get(overlayId)) {
  customElements.define(overlayId, ErrorOverlay);
}
