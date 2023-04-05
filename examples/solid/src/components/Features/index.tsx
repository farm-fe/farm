import { type Component, type JSXElement, For } from 'solid-js';

import rocket from '../../../assets/rocket.svg';
import toolbox from '../../../assets/toolbox.svg';
import plug from '../../../assets/plug.svg';
import './index.css';

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
    ),
  },
  {
    title: 'Rich Features',
    Img: toolbox,
    description: (
      <>
        Farm support compiling Html, Css, Js/Jsx/Ts/Tsx, Static Assets natively,
        support lazy compiling, partial bundling and more.
      </>
    ),
  },
  {
    title: 'Fully Pluggable',
    Img: plug,
    description: (
      <>
        Everything inside Farm is powered by plugins, you can achieve anything
        you want by writing a plugin. Support both Rust and Js plugins.
      </>
    ),
  },
];

const Feature: Component<{
  Img: string;
  title: string;
  description: JSXElement;
}> = ({ Img, title, description }) => {
  return (
    <div class="col background">
      <div class="text--center">
        <img src={Img} class="featureSvg" />
      </div>
      <div class="text--center">
        <h3>{title}</h3>
        <p style={{ color: '#333' }}>{description}</p>
      </div>
    </div>
  );
};

export const HomepageFeatures: Component = () => {
  return (
    <section class="features">
      <div class="container">
        <div class="row">
          <For each={FeatureList}>{(props) => <Feature {...props} />}</For>
        </div>
      </div>
    </section>
  );
};
