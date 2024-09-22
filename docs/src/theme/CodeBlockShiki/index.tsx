import React, { useEffect, useState } from 'react';
import { codeToHtml } from 'shiki';
import styles from './index.module.css'

const CodeBlock = ({ children, className, metastring }) => {
  const [highlightedCode, setHighlightedCode] = useState('');
  const [copied, setCopied] = useState(false);
  const language = className ? className.replace(/language-/, '') : 'javascript';
  const title = metastring ? metastring.match(/title="([^"]+)"/) : null;
  const fileName = title ? title[1] : null;
  const highlightLines = metastring ? metastring.match(/{([\d,-]+)}/) : null;

  useEffect(() => {
    const highlight = async () => {
      let lines = [];
      if (highlightLines) {
        lines = highlightLines[1].split(',').flatMap(range => {
          const [start, end] = range.split('-').map(Number);
          return end ? Array.from({ length: end - start + 1 }, (_, i) => start + i) : [start];
        });
      }

      const highlighted = await codeToHtml(children, {
        lang: language,
        themes: {
          light: 'vitesse-light',
          dark: 'vitesse-dark'
        },
        transformers: [{
          name: 'line-highlight',
          code(node) {
            let lineIndex = 0;
            node.children.forEach((child) => {
              if (child.type === 'element' && child.tagName === 'span' && child.properties.class === 'line') {
                lineIndex++;
                if (lines.includes(lineIndex)) {
                  child.properties.class = 'line highlighted-line';
                }
              }
            });
            return node;
          }
        }]
      });

      setHighlightedCode(highlighted);
    };

    highlight();
  }, [children, language, highlightLines]);

  const copyCode = () => {
    navigator.clipboard.writeText(children).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  };

  return (
    <div className={styles['shiki-wrapper']}>
      <div className={styles['code-header']}>
        {fileName && <div className={styles['code-title']}>{fileName}
        </div>}
        <button className={styles['copy-button']} onClick={copyCode}>
          {copied ? (
            <svg className={styles['copy-icon']} viewBox="0 0 24 24">
              <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" />
            </svg>
          ) : (
            <svg className={styles['copy-icon']} viewBox="0 0 24 24">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z" />
            </svg>
          )}
          <span>{copied ? 'Copy' : 'Copy'}</span>
        </button>
      </div>
      <div className={styles['code-content']} dangerouslySetInnerHTML={{ __html: highlightedCode }} />
    </div>
  );
};

export default CodeBlock;
