import React from "react";
import clsx from "clsx";
import Translate, { translate } from "@docusaurus/Translate";
// import Image from "@theme/IdealImage";
import Rocket from "@site/static/img/rocket.png";
import Plug from "@site/static/img/plug.png";
import FeaturePng from "@site/static/img/feature.png";
import Box from "@site/static/img/box.png";
import Compatibility from "@site/static/img/compatible.png";
import Consistency from "@site/static/img/consistency.png";
import styles from "./styles.module.css";
const FeatureList = [
  {
    title: <Translate>Extremely Fast</Translate>,
    Img: Rocket,
    description: (
      <Translate>
        Written in Rust, start a React / Vue project in milliseconds and perform
        an HMR update within 10ms for most situations.
      </Translate>
    ),
    className:
      "w-full rounded-lg sm:block sm:col-span-2 md:col-span-1 lg:row-start-2 lg:col-span-2",
  },
  {
    title: <Translate>Incremental Building</Translate>,
    Img: Box,
    description: (
      <Translate>
        Incremental Building: Support persistent cache, module level cache
        enabled by default, any module won't be compiled twice until it's
        changed!
      </Translate>
    ),
    className:
      "w-full rounded-lg sm:block sm:col-span-2 md:col-span-1 lg:row-start-2 lg:col-span-2",
  },
  {
    title: <Translate>Rich Features</Translate>,
    Img: FeaturePng,
    description: (
      <Translate>
        Farm supports compiling HTML, CSS, CSS Modules, Js/Jsx/Ts/Tsx, JSON,
        Static Assets out of the box, supports Sass, Less, PostCSS, Vue, React,
        Solid by way of official plugins, supports lazy compiling, partial
        bundling and more
      </Translate>
    ),
    className:
      "w-full rounded-lg sm:block sm:col-span-2 md:col-span-1 lg:row-start-2 lg:col-span-2",
  },
  {
    title: <Translate>Fully Pluggable and Vite Compatible</Translate>,
    Img: Plug,
    description: (
      <Translate>
        Everything inside Farm is powered by plugins, Supports both Rust and
        JavaScript plugins. Support Vite plugins out of box.
      </Translate>
    ),
    className:
      " w-full flex h-52 rounded-lg md:block lg:row-start-2 lg:col-span-2 lg:h-auto",
  },
  {
    title: <Translate>Partial Bundling</Translate>,
    Img: Box,
    description: (
      <Translate>
        Partial Bundling: Bundle your project into a few reasonable bundles,
        speeding up resource loading without losing caching granularity.
      </Translate>
    ),
    className:
      "w-full rounded-lg sm:block sm:col-span-2 md:col-span-1 lg:row-start-2 lg:col-span-2",
  },
  {
    title: <Translate>Consistency and Compatibility</Translate>,
    Img: Compatibility,
    description: (
      <Translate>
        What you see in development will be the same as what you get in
        production. Supports both legacy (ES5) and modern browsers.
      </Translate>
    ),
    className:
      " w-full flex h-52 rounded-lg md:block lg:row-start-2 lg:col-span-2 lg:h-auto",
  },
];

function Feature({ Img, title, description, className }) {
  return (
    <div
      className={clsx(
        "rounded-lg shadow-lg",
        styles.card,
        styles["card-container"],
        className
      )}
    >
      <div
        className={clsx(
          "flex items-center flex-col",
          styles["card-container-content"]
        )}
      >
        <div
          className={clsx(
            "flex items-center justify-center absolute",
            styles.backgroundImage
          )}
        >
          <img
            src={Img}
            className={clsx("text--center w-20 h-20")}
            role="img"
          />
        </div>
        <img src={Img} className={clsx("text--center w-16 h-16")} role="img" />
        <div className="p-6 flex-grow flex-shrink">
          <h3 className="text-lg font-bold mt-4 mb-2">{title}</h3>
          <p className="text-base">{description}</p>
        </div>
      </div>
    </div>
  );
}

export default function FeatureSection() {
  return (
    <section>
      <div className="max-w-7xl mx-auto flex">
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-8">
          {FeatureList.map((feature, index) => (
            <Feature
              key={index}
              {...feature}
              className="flex-grow flex-shrink my-4"
            />
          ))}
        </div>
      </div>
    </section>
  );
}
