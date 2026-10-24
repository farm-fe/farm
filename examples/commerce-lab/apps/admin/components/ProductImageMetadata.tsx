import React, { useEffect, useRef, useState } from 'react';
import home from '../assets/product-home.svg?dom';
import outdoor from '../assets/product-outdoor.svg?dom';
import studio from '../assets/product-studio.svg?dom';
import travel from '../assets/product-travel.svg?dom';

const images: Record<string, HTMLImageElement> = { Home: home, Outdoor: outdoor, Studio: studio, Travel: travel };
export function ProductImageMetadata({ category, name }: { category: string; name: string }) {
  const container = useRef<HTMLDivElement>(null);
  const [status, setStatus] = useState('Loading image');
  const [dimensions, setDimensions] = useState('');
  const source = images[category] ?? home;
  useEffect(() => {
    const image = source.cloneNode(false) as HTMLImageElement;
    image.alt = `${name} category illustration`;
    image.width = 480;
    image.style.maxWidth = '100%';
    image.style.height = 'auto';
    const loaded = () => { setStatus('Image loaded'); setDimensions(`${image.naturalWidth} × ${image.naturalHeight}`); };
    const failed = () => { setStatus('Product image could not load.'); setDimensions(''); };
    setStatus('Loading image');
    setDimensions('');
    image.addEventListener('load', loaded);
    image.addEventListener('error', failed);
    container.current?.appendChild(image);
    if (image.complete) image.naturalWidth > 0 ? loaded() : failed();
    return () => {
      image.removeEventListener('load', loaded);
      image.removeEventListener('error', failed);
      image.remove();
    };
  }, [source, name]);
  return <div><div ref={container} data-testid="image-dom-preview" />
    <p data-testid="image-dom-status" role={status.includes('could not') ? 'alert' : 'status'}>{status}</p>
    {dimensions && <p>Natural size: {dimensions}</p>}
  </div>;
}
