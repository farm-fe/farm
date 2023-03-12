import React from 'react';
import clsx from 'clsx';
import './index.css';

import rocket from '../../../assets/rocket.svg';
import toolbox from '../../../assets/toolbox.svg';
import plug from '../../../assets/plug.svg';

const FeatureList = [
  {
    title: 'Super Fast',
    Img: rocket,
    description: (
      <>
        Farm's compiler is written in Rust, with multi-threading,
        lazy/asynchronous compilation and persist caching, Farm can start a
        project in milliseconds, perform a HMR update within 10ms.
      </>
    )
  },
  {
    title: 'Rich Features',
    Img: toolbox,
    description: (
      <>
        Farm support compiling Html, Css, Js/Jsx/Ts/Tsx, Static Assets natively,
        support lazy compiling, partial bundling and more.
      </>
    )
  },
  {
    title: 'Fully Pluggable',
    Img: plug,
    description: (
      <>
        Everything inside Farm is powered by plugins, you can achieve anything
        you want by writing a plugin. Support both Rust and Js plugins.
      </>
    )
  }
];

function Feature({
  Img,
  title,
  description
}: {
  Img: string;
  title: string;
  description: React.ReactNode;
}) {
  return (
    <div className={clsx('col background')}>
      <div className="text--center">
        <img src={Img} className="featureSvg" />
      </div>
      <div className="text--center">
        <h3>{title}</h3>
        <p style={{ color: '#333' }}>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures() {
  return (
    <section className="features">
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
