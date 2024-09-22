import React, { useEffect, useState } from 'react';
import { codeToHtml } from 'shiki';

const CodeBlock = ({ children, className, metastring }) => {
  const [highlightedCode, setHighlightedCode] = useState('');
  const language = className ? className.replace(/language-/, '') : 'text';
  const title = metastring ? metastring.match(/title="([^"]+)"/) : null;
  const highlightLines = metastring ? metastring.match(/{([\d,-]+)}/) : null;

  useEffect(() => {
    const highlight = async () => {
      let lines: any[] = [];
      if (highlightLines) {
        lines = highlightLines[1].split(',').flatMap(range => {
          const [start, end] = range.split('-').map(Number);
          return end ? Array.from({ length: end - start + 1 }, (_, i) => start + i) : [start];
        });
      }

      const highlighted = await codeToHtml(children, {
        lang: language,
        theme: 'vitesse-light',
        transformers: [{
          name: 'line-highlight',
          code(node) {
            let lineIndex: any = 0;
            node.children.forEach((child, index) => {
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

  return (
    <div className="shiki-wrapper">
      {title && <div className="code-title">{title[1]}</div>}
      <div dangerouslySetInnerHTML={{ __html: highlightedCode }} />
    </div>
  );
};

export default CodeBlock;
