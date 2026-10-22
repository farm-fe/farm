import React, { useState } from 'react';
import { useInView } from 'react-intersection-observer';
import { SiteText } from '../../site-text';
import { ProgressBar } from './ProgressBar';

const BENCHMARK_DATA = {
  "ColdStart": [
    {
      "name": "Farm",
      "time": 0.396
    },
    {
      "name": "RsBuild",
      "time": 0.468
    },
    {
      "name": "Vite",
      "time": 1.7
    },
    {
      "name": "Webpack",
      "time": 2.078
    }
  ],
  "HotStart": [
    {
      "name": "Farm",
      "time": 0.273
    },
    {
      "name": "RsBuild",
      "time": 0.468
    },
    {
      "name": "Vite",
      "time": 1.426
    },
    {
      "name": "Webpack",
      "time": 0.945
    }
  ],
  "HmrRoot": [
    {
      "name": "Farm",
      "time": 0.018
    },
    {
      "name": "RsBuild",
      "time": 0.087
    },
    {
      "name": "Vite",
      "time": 0.022
    },
    {
      "name": "Webpack",
      "time": 0.532
    }
  ],
  "HmrLeaf": [
    {
      "name": "Farm",
      "time": 0.013
    },
    {
      "name": "RsBuild",
      "time": 0.074
    },
    {
      "name": "Vite",
      "time": 0.011
    },
    {
      "name": "Webpack",
      "time": 0.165
    }
  ],
  "ColdBuild": [
    {
      "name": "Farm",
      "time": 0.313
    },
    {
      "name": "RsBuild",
      "time": 0.363
    },
    {
      "name": "Vite",
      "time": 1.543
    },
    {
      "name": "Webpack",
      "time": 4.128
    }
  ],
  "HotBuild": [
    {
      "name": "Farm",
      "time": 0.16
    },
    {
      "name": "RsBuild",
      "time": 0.363
    },
    {
      "name": "Vite",
      "time": 1.54
    },
    {
      "name": "Webpack",
      "time": 0.527
    }
  ]
};
export default function Benchmark() {
  const scenes = [
    { label: <SiteText id="benchmark.coldStart" />, value: 'ColdStart' },
    { label: <SiteText id="benchmark.hotStart" />, value: 'HotStart' },
    { label: <SiteText id="benchmark.hmrRoot" />, value: 'HmrRoot' },
    { label: <SiteText id="benchmark.hmrLeaf" />, value: 'HmrLeaf' },
    { label: <SiteText id="benchmark.coldBuild" />, value: 'ColdBuild' },
    { label: <SiteText id="benchmark.hotBuild" />, value: 'HotBuild' }
  ];
  const [activeScene, setActiveScene] = useState('ColdStart');
  const { ref, inView } = useInView({ triggerOnce: true, threshold: 0.1 });
  const entries = BENCHMARK_DATA[activeScene];
  return <div ref={ref} className="farm-home-benchmark-chart">
    <div className="farm-home-benchmark-tabs" role="tablist" aria-label="Benchmark">
      {scenes.map((scene, index) => <button key={scene.value} type="button" role="tab" id={`benchmark-tab-${scene.value}`} aria-selected={activeScene === scene.value} aria-controls="farm-benchmark-panel" tabIndex={activeScene === scene.value ? 0 : -1} onClick={() => setActiveScene(scene.value)} onKeyDown={event => {
        const next = event.key === 'ArrowRight' ? (index + 1) % scenes.length : event.key === 'ArrowLeft' ? (index + scenes.length - 1) % scenes.length : event.key === 'Home' ? 0 : event.key === 'End' ? scenes.length - 1 : null;
        if (next == null) return;
        event.preventDefault(); setActiveScene(scenes[next].value);
        event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>('button')[next]?.focus();
      }}>{scene.label}</button>)}
    </div>
    <div id="farm-benchmark-panel" role="tabpanel" aria-labelledby={`benchmark-tab-${activeScene}`}>
      {entries.map(info => <div className="farm-home-benchmark-row" key={info.name}><span>{info.name}</span><ProgressBar value={info.time} max={Math.max(...entries.map(row => row.time))} animate={inView} contained /></div>)}
    </div>
    <a className="farm-home-benchmark-details" href="https://github.com/farm-fe/performance-compare" target="_blank" rel="noopener noreferrer">📈 See Benchmark Details</a>
  </div>;
}
