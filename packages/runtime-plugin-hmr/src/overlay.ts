import hljs from 'highlight.js';
import {
  extractErrorMessage,
  parseIfJSON,
  splitErrorMessage,
  stripAnsi
} from './utils';

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
  --red: #cf1322;
  --brand-color: #9f1a8f;
  --window-background: #ffffff;
  --window-color: #ccc;
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
  width: 1000px;
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
  max-height: 500px;
  font-size: 14px;
  line-height: 1.3;
  font-weight: 600;
  white-space: pre-wrap;
  overflow-y: auto;
}

.message {
    scrollbar-width: none;  /* Firefox */
    -ms-overflow-style: none;  /* IE and Edge */
}

.message::-webkit-scrollbar {
    display: none;  /* Chrome, Safari, and Opera */
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
.message-container {
  padding: 10px 10px;
}


.code-block {
  background-color: #f8f8f8;
  border-radius: 3px;
  padding: 10px;
  margin: 10px 0;
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
.message-container {
}
pre code.hljs {
  display: block;
  overflow-x: auto;
  padding: 1em
}
code.hljs {
  padding: 3px 5px
}
/*

Atom One Dark by Daniel Gamage
Original One Dark Syntax theme from https://github.com/atom/one-dark-syntax

base:    #282c34
mono-1:  #abb2bf
mono-2:  #818896
mono-3:  #5c6370
hue-1:   #56b6c2
hue-2:   #61aeee
hue-3:   #c678dd
hue-4:   #98c379
hue-5:   #e06c75
hue-5-2: #be5046
hue-6:   #d19a66
hue-6-2: #e6c07b

*/
.hljs {
  color: #abb2bf;
  background: #282c34
}
.hljs-comment,
.hljs-quote {
  color: #5c6370;
  font-style: italic
}
.hljs-doctag,
.hljs-keyword,
.hljs-formula {
  color: #c678dd
}
.hljs-section,
.hljs-name,
.hljs-selector-tag,
.hljs-deletion,
.hljs-subst {
  color: #e06c75
}
.hljs-literal {
  color: #56b6c2
}
.hljs-string,
.hljs-regexp,
.hljs-addition,
.hljs-attribute,
.hljs-meta .hljs-string {
  color: #98c379
}
.hljs-attr,
.hljs-variable,
.hljs-template-variable,
.hljs-type,
.hljs-selector-class,
.hljs-selector-attr,
.hljs-selector-pseudo,
.hljs-number {
  color: #d19a66
}
.hljs-symbol,
.hljs-bullet,
.hljs-link,
.hljs-meta,
.hljs-selector-id,
.hljs-title {
  color: #61aeee
}
.hljs-built_in,
.hljs-title.class_,
.hljs-class .hljs-title {
  color: #e6c07b
}
.hljs-emphasis {
  font-style: italic
}
.hljs-strong {
  font-weight: bold
}
.hljs-link {
  text-decoration: underline
}

.code-block-wrapper {
  background-color: #282c34; /* Atom One Dark 背景色 */
  border-radius: 6px;
  padding: 16px;
  margin: 10px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.code-block-wrapper pre {
  margin: 0;
}

.code-block-wrapper code {
  font-family: 'Consolas', 'Monaco', 'Andale Mono', 'Ubuntu Mono', monospace;
  font-size: 14px;
  line-height: 1.5;
}
.alert {
  box-sizing: border-box;
  margin: 8px 0;
  padding: 0;
  color: rgba(0, 0, 0, 0.85);
  font-size: 14px;
  font-variant: tabular-nums;
  line-height: 1.5715;
  list-style: none;
  font-feature-settings: 'tnum';
  position: relative;
  display: flex;
  align-items: flex-start;
  padding: 8px 15px;
  word-wrap: break-word;
  border-radius: 4px;
}

.alert-error {
  background-color: #fff2f0;
  border: 1px solid #ffccc7;
}

.alert-icon {
  margin-right: 14px;
  font-size: 14px;
  margin-top: 2px;
}

.alert-error .alert-icon {
  color: #ff4d4f;
}

.alert-content {
  flex: 1;
  min-width: 0;
}

.alert-message {
  color: rgba(0, 0, 0, 0.85);
  font-size: 14px;
}

.alert-description {
  font-size: 14px;
  line-height: 22px;
}

.alert-error .alert-message {
  color: #cf1322;
}

.alert-error .alert-description {
  color: rgba(0, 0, 0, 0.75);
}

.alert-warn {
  background-color: #fffbe6;
  border: 1px solid #ffe58f;
}

.alert-warn .alert-icon {
  color: #faad14;
}

.alert-warn .alert-message {
  color: #d46b08;
}

.alert-warn .alert-description {
  color: rgba(0, 0, 0, 0.75);
}

.terminal-block {
  background-color: #1e1e1e;
  border-radius: 6px;
  overflow: hidden;
  margin: 4px 0;
}

.file-info {
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: #2d2d2d;
  color: #d4d4d4;
  padding: 8px 12px;
  font-family: 'Consolas', 'Monaco', 'Andale Mono', 'Ubuntu Mono', monospace;
  font-size: 12px;
}

.file-info-left {
  display: flex;
  align-items: center;
}

.file-info-left svg {
  margin-right: 8px;
}

</style>
<div class="backdrop" part="backdrop">
  <div class="window" part="window">
    <div class="message-container" part="message-container">
    </div>
    <div class="footer">
      <div class="tip" part="tip">
        Click outside, press <kbd>Esc</kbd> key, or fix the code to dismiss.<br />
      </div>
    </div>
  </div>
</div>
`;

const errorAlert = (
  message: string,
  description: string,
  _type: string
) => /*html*/ `
    <div class="alert alert-error">
      <span class="alert-icon">
        <svg viewBox="64 64 896 896" focusable="false" data-icon="close-circle" width="1.2em" height="1.2em" fill="currentColor" aria-hidden="true">
          <path d="M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm165.4 618.2l-66-.3L512 563.4l-99.3 118.4-66.1.3c-4.4 0-8-3.5-8-8 0-1.9.7-3.7 1.9-5.2l130.1-155L340.5 359a8.32 8.32 0 01-1.9-5.2c0-4.4 3.6-8 8-8l66.1.3L512 464.6l99.3-118.4 66-.3c4.4 0 8 3.5 8 8 0 1.9-.7 3.7-1.9 5.2L553.5 514l130 155c1.2 1.5 1.9 3.3 1.9 5.2 0 4.4-3.6 8-8 8z"></path>
        </svg>
      </span>
      <div class="alert-content">
        <div class="alert-message">${message}</div>
        <div class="alert-description">Failed to compile: ${description}</div>
      </div>
    </div>
`;

const warnAlert = (description: string, type = 'warn') => /*html*/ `
    <div class="alert alert-${type}">
      <span class="alert-icon">
        ${getAlertIcon(type)}
      </span>
      <div class="alert-content">
        <div class="alert-description">${description.replace(/\n/g, '<br>')}</div>
      </div>
    </div>
`;

const fileRE = /(?:[a-zA-Z]:\\|\/).*?:\d+:\d+/g;
// const codeframeRE = /^(?:>?\s*\d+\s+\|.*|\s+\|\s*\^.*)\r?\n/gm;

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

    const messages = parseIfJSON(err.message);

    this.renderMessages(messages, links);

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

  setMessageText(element: HTMLElement, text: string, linkFiles: boolean) {
    if (!linkFiles) {
      element.textContent = text;
    } else {
      element.innerHTML = '';
      let lastIndex = 0;
      text.replace(fileRE, (file, index) => {
        if (index > lastIndex) {
          element.appendChild(
            document.createTextNode(text.substring(lastIndex, index))
          );
        }
        const link = document.createElement('a');
        link.textContent = file;
        link.className = 'file-link';
        link.onclick = () => {
          fetch(`${base}__open-in-editor?file=${encodeURIComponent(file)}`);
        };
        element.appendChild(link);
        lastIndex = index + file.length;
        return file;
      });
      if (lastIndex < text.length) {
        element.appendChild(document.createTextNode(text.substring(lastIndex)));
      }
    }
  }

  highlightCode(code: string, language = 'javascript') {
    if (language) {
      return hljs.highlight(code, { language }).value;
    }
    return hljs.highlightAuto(code).value;
  }

  renderMessages(messages: any[], _link: any) {
    const messageContainer = this.root.querySelector('.message-container')!;
    messageContainer.innerHTML = '';
    if (typeof messages === 'string') {
      const messageElement = document.createElement('div');
      const messageBody = document.createElement('div');
      messageBody.className = 'message';
      const terminalBlock = document.createElement('div');
      const highlightedCode = this.highlightCode(extractErrorMessage(messages));
      terminalBlock.className = 'terminal-block';

      const fileInfo = document.createElement('div');
      fileInfo.className = 'file-info';

      const fileInfoRight = document.createElement('div');
      const fileInfoLeft = document.createElement('div');
      fileInfoRight.className = 'file-info-right';
      fileInfoRight.innerHTML = `
        <svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-square-terminal">
          <path d="m7 11 2-2-2-2"/>
          <path d="M11 13h4"/>
          <rect width="18" height="18" x="3" y="3" rx="2" ry="2"/>
        </svg>
      `;

      fileInfoLeft.className = 'file-info-left';
      fileInfoLeft.textContent = '';

      fileInfo.appendChild(fileInfoLeft);
      fileInfo.appendChild(fileInfoRight);

      const codeBlockWrapper = document.createElement('div');
      codeBlockWrapper.className = 'code-block-wrapper';
      codeBlockWrapper.innerHTML = highlightedCode;

      terminalBlock.appendChild(fileInfo);
      terminalBlock.appendChild(codeBlockWrapper);
      messageBody.appendChild(terminalBlock);
      messageElement.appendChild(messageBody);
      messageContainer.appendChild(messageElement);
      return;
    }

    messages.forEach((msg) => {
      const messageElement = document.createElement('div');
      messageElement.className = 'error-message';
      msg = parseIfJSON(msg);

      if (msg.type) {
        const TypeError = document.createElement('span');
        TypeError.className = 'type-error';
        TypeError.textContent = msg.type;
        const TypeCodeError = document.createElement('div');
        TypeCodeError.innerHTML = errorAlert(msg.type, msg.id, msg.type);
        messageElement.appendChild(TypeCodeError);
      }

      // if (msg.plugin) {
      //   const pluginElement = document.createElement('span');
      //   pluginElement.className = 'plugin';
      //   pluginElement.textContent = `[plugin:${msg.plugin}] `;
      //   messageElement.appendChild(pluginElement);
      // }

      const messageBody = document.createElement('div');
      messageBody.className = 'message';

      const splitMessage = splitErrorMessage(msg);

      console.error(splitMessage.errorInfo);

      if (splitMessage.codeBlocks && splitMessage.codeBlocks.length > 0) {
        splitMessage.codeBlocks.forEach((codeBlock, blockIndex) => {
          const terminalBlock = document.createElement('div');
          const highlightedCode = this.highlightCode(codeBlock);

          terminalBlock.className = 'terminal-block';

          const fileInfo = document.createElement('div');
          fileInfo.className = 'file-info';

          const fileInfoRight = document.createElement('div');
          const fileInfoLeft = document.createElement('div');
          fileInfoRight.className = 'file-info-right';
          fileInfoRight.innerHTML = `
            <svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-square-terminal">
              <path d="m7 11 2-2-2-2"/>
              <path d="M11 13h4"/>
              <rect width="18" height="18" x="3" y="3" rx="2" ry="2"/>
            </svg>
          `;

          fileInfoLeft.className = 'file-info-left';
          fileInfoLeft.textContent = splitMessage.idCodeLines[blockIndex];

          fileInfo.appendChild(fileInfoLeft);
          fileInfo.appendChild(fileInfoRight);

          const codeBlockWrapper = document.createElement('div');
          codeBlockWrapper.className = 'code-block-wrapper';
          codeBlockWrapper.innerHTML = highlightedCode;

          terminalBlock.appendChild(fileInfo);
          terminalBlock.appendChild(codeBlockWrapper);
          messageBody.appendChild(terminalBlock);
        });
      } else if (splitMessage.errorInfo) {
        const terminalBlock = document.createElement('div');

        const highlightedCode = this.highlightCode(
          extractErrorMessage(splitMessage.frame)
        );

        terminalBlock.className = 'terminal-block';

        const fileInfo = document.createElement('div');
        fileInfo.className = 'file-info';

        const fileInfoRight = document.createElement('div');
        const fileInfoLeft = document.createElement('div');
        fileInfoRight.className = 'file-info-right';
        fileInfoRight.innerHTML = `
          <svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-square-terminal">
            <path d="m7 11 2-2-2-2"/>
            <path d="M11 13h4"/>
            <rect width="18" height="18" x="3" y="3" rx="2" ry="2"/>
          </svg>
        `;

        fileInfoLeft.className = 'file-info-left';
        fileInfoLeft.textContent = `Error Info ${
          msg.plugin ? `[plugin: ${msg.plugin}]` : ''
        }`;

        fileInfo.appendChild(fileInfoLeft);
        fileInfo.appendChild(fileInfoRight);

        const codeBlockWrapper = document.createElement('div');
        codeBlockWrapper.className = 'code-block-wrapper';
        codeBlockWrapper.innerHTML = highlightedCode;

        terminalBlock.appendChild(fileInfo);
        terminalBlock.appendChild(codeBlockWrapper);
        messageBody.appendChild(terminalBlock);
      }

      messageElement.appendChild(messageBody);

      if (msg.frame) {
        const messageElement = document.createElement('div');
        const messageBody = document.createElement('div');
        messageBody.className = 'message';
        const terminalBlock = document.createElement('div');
        const highlightedCode = this.highlightCode(
          extractErrorMessage(stripAnsi(msg.frame))
        );
        terminalBlock.className = 'terminal-block';

        const fileInfo = document.createElement('div');
        fileInfo.className = 'file-info';

        const fileInfoRight = document.createElement('div');
        const fileInfoLeft = document.createElement('div');
        fileInfoRight.className = 'file-info-right';
        fileInfoRight.innerHTML = `
          <svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-square-terminal">
            <path d="m7 11 2-2-2-2"/>
            <path d="M11 13h4"/>
            <rect width="18" height="18" x="3" y="3" rx="2" ry="2"/>
          </svg>
        `;

        fileInfoLeft.className = 'file-info-left';
        fileInfoLeft.textContent = '';

        fileInfo.appendChild(fileInfoLeft);
        fileInfo.appendChild(fileInfoRight);

        const codeBlockWrapper = document.createElement('div');
        codeBlockWrapper.className = 'code-block-wrapper';
        codeBlockWrapper.innerHTML = highlightedCode;

        terminalBlock.appendChild(fileInfo);
        terminalBlock.appendChild(codeBlockWrapper);
        messageBody.appendChild(terminalBlock);
        messageElement.appendChild(messageBody);
        messageContainer.appendChild(messageElement);
      }

      // if (msg.stack) {
      //   const stack = document.createElement('pre');
      //   stack.className = 'stack';
      //   this.setMessageText(stack, msg.stack, links);
      //   messageElement.appendChild(stack);
      // }

      if (msg.cause) {
        const causeElement = document.createElement('div');
        causeElement.innerHTML = warnAlert(msg.cause);
        messageElement.appendChild(causeElement);
      }

      messageContainer.appendChild(messageElement);
    });
  }

  close(): void {
    this.parentNode?.removeChild(this);
    document.removeEventListener('keydown', this.closeOnEsc);
  }
}

export const overlayId = 'farm-error-overlay';
const { customElements } = globalThis; // Ensure `customElements` is defined before the next line.
if (customElements && !customElements.get(overlayId)) {
  customElements.define(overlayId, ErrorOverlay);
}

function getAlertIcon(type: string) {
  switch (type) {
    case 'error':
      return `<svg viewBox="64 64 896 896" focusable="false" data-icon="close-circle" width="1.2em" height="1.2em" fill="currentColor" aria-hidden="true">
        <path d="M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm165.4 618.2l-66-.3L512 563.4l-99.3 118.4-66.1.3c-4.4 0-8-3.5-8-8 0-1.9.7-3.7 1.9-5.2l130.1-155L340.5 359a8.32 8.32 0 01-1.9-5.2c0-4.4 3.6-8 8-8l66.1.3L512 464.6l99.3-118.4 66-.3c4.4 0 8 3.5 8 8 0 1.9-.7 3.7-1.9 5.2L553.5 514l130 155c1.2 1.5 1.9 3.3 1.9 5.2 0 4.4-3.6 8-8 8z"></path>
      </svg>`;
    case 'warn':
      return `<svg viewBox="64 64 896 896" focusable="false" data-icon="exclamation-circle" width="1.2em" height="1.2em" fill="currentColor" aria-hidden="true">
        <path d="M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm-32 232c0-4.4 3.6-8 8-8h48c4.4 0 8 3.6 8 8v272c0 4.4-3.6 8-8 8h-48c-4.4 0-8-3.6-8-8V296zm32 440a48.01 48.01 0 010-96 48.01 48.01 0 010 96z"></path>
      </svg>`;
    default:
      return '';
  }
}
