import CodeBlock from '@theme-original/CodeBlock';
import type CodeBlockType from '@theme/CodeBlock';
import type { WrapperProps } from '@docusaurus/types';
import React, { useEffect, useState } from 'react';
import styles from './index.module.css'
type Props = WrapperProps<typeof CodeBlockType>;

export default function CodeBlockWrapper(props: Props): JSX.Element {
  const [highlightedCode, setHighlightedCode] = useState('');
  const [copied, setCopied] = useState(false);
  const language = props.className ? props.className.replace(/language-/, '') : 'javascript';
  const title = props.metastring ? props.metastring.match(/title="([^"]+)"/) : null;
  const fileName = title ? title[1] : null;

  const copyCode = () => {
    navigator.clipboard.writeText(props.children).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  };
  return (
    <>
      <div className={styles['shiki-wrapper']}>
        <div className={styles['code-header']}>
          {fileName && <div className={styles['code-title']}>{fileName}
          </div>}
          {/* <button className={styles['copy-button']} onClick={copyCode}>
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
          </button> */}
        </div>
        <CodeBlock className={styles['code-content']} {...props} />
      </div>
    </>
  );
}
