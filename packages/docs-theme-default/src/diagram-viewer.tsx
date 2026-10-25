import { useEffect, useRef, useState } from 'react';

/** Enhance maintained SVG links without changing their SSR or no-JS fallback. */
export function DiagramViewer() {
  const dialog = useRef<HTMLDialogElement>(null);
  const viewport = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLElement | null>(null);
  const pointers = useRef(new Map<number, { x: number; y: number }>());
  const previous = useRef<{ x: number; y: number; distance: number } | null>(null);
  const [image, setImage] = useState({ src: '', label: '', chinese: false });
  const [view, setView] = useState({ scale: 1, x: 0, y: 0 });
  const reset = () => setView({ scale: 1, x: 0, y: 0 });
  const zoom = (factor: number) => setView(v => ({ ...v, scale: Math.max(1, Math.min(6, v.scale * factor)) }));
  useEffect(() => {
    const open = (event: MouseEvent) => {
      if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
      const link = (event.target as Element).closest<HTMLAnchorElement>('a.farm-docs-diagram');
      if (!link) return;
      const url = new URL(link.href, location.href);
      if (url.origin !== location.origin || !url.pathname.endsWith('.svg')) return;
      event.preventDefault();
      trigger.current = link;
      setImage({ src: url.href, label: link.querySelector('img')?.alt || 'Diagram', chinese: document.documentElement.lang.startsWith('zh') });
      reset();
      dialog.current?.showModal();
      document.documentElement.classList.add('farm-diagram-open');
    };
    document.addEventListener('click', open);
    return () => { document.removeEventListener('click', open); document.documentElement.classList.remove('farm-diagram-open'); };
  }, []);
  const finish = () => {
    pointers.current.clear(); previous.current = null;
    document.documentElement.classList.remove('farm-diagram-open');
    trigger.current?.focus();
  };
  const close = () => dialog.current?.close();
  const c = image.chinese;
  return <dialog ref={dialog} className="farm-diagram-viewer" aria-labelledby="farm-diagram-title" onClose={finish} onClick={e => { if (e.target === dialog.current) close(); }}>
    <header><strong id="farm-diagram-title">{image.label}</strong><div role="group" aria-label={c ? '图表控制' : 'Diagram controls'}>
      <button type="button" aria-label={c ? '缩小' : 'Zoom out'} onClick={() => zoom(1 / 1.3)}>−</button>
      <output aria-live="polite">{Math.round(view.scale * 100)}%</output>
      <button type="button" aria-label={c ? '放大' : 'Zoom in'} onClick={() => zoom(1.3)}>+</button>
      <button type="button" onClick={reset}>{c ? '重置' : 'Reset'}</button>
      <button type="button" onClick={close} aria-label={c ? '关闭图表' : 'Close diagram'}>×</button>
    </div></header>
    <p>{c ? '拖动或双指缩放查看；键盘 + / − 缩放，方向键移动，0 重置，Esc 关闭。' : 'Drag or pinch to explore. Keyboard: + / − zoom, arrows pan, 0 resets, Esc closes.'}</p>
    <div ref={viewport} className="farm-diagram-viewport" tabIndex={0} aria-label={c ? '可缩放图表' : 'Zoomable diagram'}
      onKeyDown={e => {
        if (e.key === '+' || e.key === '=') zoom(1.3);
        else if (e.key === '-') zoom(1 / 1.3);
        else if (e.key === '0') reset();
        else if (e.key.startsWith('Arrow')) setView(v => ({ ...v, x: v.x + (e.key === 'ArrowLeft' ? 40 : e.key === 'ArrowRight' ? -40 : 0), y: v.y + (e.key === 'ArrowUp' ? 40 : e.key === 'ArrowDown' ? -40 : 0) }));
        else return;
        e.preventDefault();
      }}
      onPointerDown={e => { e.currentTarget.setPointerCapture(e.pointerId); pointers.current.set(e.pointerId, { x: e.clientX, y: e.clientY }); previous.current = null; }}
      onPointerMove={e => {
        if (!pointers.current.has(e.pointerId)) return;
        pointers.current.set(e.pointerId, { x: e.clientX, y: e.clientY });
        const points = [...pointers.current.values()];
        const x = points.reduce((s, p) => s + p.x, 0) / points.length;
        const y = points.reduce((s, p) => s + p.y, 0) / points.length;
        const distance = points.length > 1 ? Math.hypot(points[0].x - points[1].x, points[0].y - points[1].y) : 0;
        const last = previous.current;
        if (last) setView(v => ({ x: v.x + x - last.x, y: v.y + y - last.y, scale: Math.max(1, Math.min(6, last.distance && distance ? v.scale * distance / last.distance : v.scale)) }));
        previous.current = { x, y, distance };
      }}
      onPointerUp={e => { pointers.current.delete(e.pointerId); previous.current = null; }}
      onPointerCancel={e => { pointers.current.delete(e.pointerId); previous.current = null; }}>
      {image.src && <img src={image.src} alt={image.label} draggable={false} style={{ transform: `translate(${view.x}px, ${view.y}px) scale(${view.scale})` }} />}
    </div>
  </dialog>;
}
