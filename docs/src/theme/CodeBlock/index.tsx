import CodeBlock from '@theme-original/CodeBlock';
import type CodeBlockType from '@theme/CodeBlock';
import type { WrapperProps } from '@docusaurus/types';
import React, { useEffect, useState } from 'react';
import styles from './index.module.css'
import { codeToHtml } from 'shiki';
import useIsBrowser from '@docusaurus/useIsBrowser';
import { useColorMode } from '@docusaurus/theme-common';
import { svgs } from './svg';
type Props = WrapperProps<typeof CodeBlockType>;

export default function CodeBlockWrapper(props: Props): JSX.Element {
  const [highlightedCode, setHighlightedCode] = useState('');
  const [copied, setCopied] = useState(false);
  const language = props.className ? props.className.replace(/language-/, '') : 'javascript';
  const title = props.metastring ? props.metastring.match(/title="([^"]+)"/) : null;
  const fileName = title ? title[1] : null;
  const highlightLines = props.metastring ? props.metastring.match(/{([\d,-]+)}/) : null;
  const { colorMode } = useColorMode();
  const iconMap = {
    ts: svgs.find(svg => svg.name === "ts")?.content,
    js: svgs.find(svg => svg.name === "js")?.content,
    css: svgs.find(svg => svg.name === "css")?.content,
    json: svgs.find(svg => svg.name === "json")?.content,
    txt: svgs.find(svg => svg.name === "text")?.content,
    vue: svgs.find(svg => svg.name === "vue")?.content,
  };
  const getFileIcon = (fileName) => {
    if (!fileName) return null;
    const extension = fileName.split('.').pop().toLowerCase();
    const svgContent = iconMap[extension];
    return svgContent ? renderSvg(svgContent, styles['file-icon']) : null;
  };
  const renderSvg = (content, className) => (
    <span
      className={className}
      dangerouslySetInnerHTML={{ __html: content }}
    />
  );
  useEffect(() => {
    const highlight = async () => {
      let lines = [];
      if (highlightLines) {
        lines = highlightLines[1].split(',').flatMap(range => {
          const [start, end] = range.split('-').map(Number);
          return end ? Array.from({ length: end - start + 1 }, (_, i) => start + i) : [start];
        });
      }

      const highlighted = await codeToHtml(props.children, {
        lang: language,
        theme: colorMode === 'dark' ? 'vitesse-dark' : 'vitesse-light',
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
  }, [props.children, colorMode, language, highlightLines]);
  const copyCode = () => {
    navigator.clipboard.writeText(props.children).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  };
  const hiddenCopy = language === 'bash' || language === 'shell';
  return (
    <>
      <div className={styles['shiki-wrapper']}>
        {!hiddenCopy && <div className={styles['code-header']}>
          {fileName && (
            <div className={styles['code-title']}>
              {getFileIcon(fileName)}
              {fileName}
            </div>
          )}
          {/* <button className={styles['copy-button']} onClick={copyCode}>
            {copied ? (
              <svg className={`${styles['copy-icon']} ${styles['copy-success']}`} viewBox="0 0 24 24">
                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" />
              </svg>
            ) : (
              <svg className={styles['copy-icon']} viewBox="0 0 24 24">
                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z" />
              </svg>
            )}
          </button> */}
        </div>}
        <div className={styles['hover-button']}>
          <button className={styles['copy-button']} onClick={copyCode}>
            {copied ? (
              <svg className={`${styles['copy-icon']} ${styles['copy-success']}`} viewBox="0 0 24 24">
                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" />
              </svg>
            ) : (
              <svg className={styles['copy-icon']} viewBox="0 0 24 24">
                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z" />
              </svg>
            )}
          </button>
          <div className={styles['code-content']} dangerouslySetInnerHTML={{ __html: highlightedCode }} />
        </div>
      </div>
    </>
  );
}

