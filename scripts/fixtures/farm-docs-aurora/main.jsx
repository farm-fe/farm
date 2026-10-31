import React from 'react';
import { createRoot } from 'react-dom/client';
import { AuroraBackground } from '../../../website/src/components/ui/aurora-back';

createRoot(document.getElementById('root')).render(<>
  <AuroraBackground data-testid="aurora" />
  <div id="filter-parent" className="blur-[10px]"><div id="filter-child" className="invert" /></div>
  <div id="filter-composite" className="blur-[10px] invert" />
</>);
