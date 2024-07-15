import React from 'react';
import { FarmCard } from '../card/index';
import { ButtonAction } from '../button/index';
import './index.css';

const images = import.meta.glob(
  ['../../../assets/*.svg', '../../../assets/*.png'],
  { as: 'url', eager: true }
);
const normalizedImages: Record<string, string> = Object.fromEntries(
  Object.entries(images).map(([key, value]) => {
    const regex = new RegExp('../../../assets/(.+)\\.(svg|png)');
    const newKey = key.match(regex)?.[1];
    if (newKey) {
      return [newKey, value as string];
    } else {
      return [key, value as string];
    }
  })
);
const { logo, feature, light, plugin } = normalizedImages;

export function Welcome() {
  return (
    <div className="farm-container">
      <img className="logo" src={logo} alt="" />
      <FarmCard>
        <div className="main-desc">
          <h2 className="main-sub-title">
            Get started With
            <span className="main-content"> React + Farm</span>
          </h2>
          <span className="main-content">
            Super fast web building tool written in Rust.
          </span>
        </div>
      </FarmCard>
      <div className="farm-desc">
        <div className="farm-desc-left">
          <FarmCard>
            <div className="container-box">
              <img src={feature} alt="" />
              <div className="rich-desc">
                <span className="sub-title">Rich Features</span>
                <span className="rich-content">
                  Farm support compiling Html, Css, Js/Jsx/Ts/Tsx, Static Assets
                  natively, support lazy compiling, partial bundling and more.
                </span>
              </div>
            </div>
          </FarmCard>
          <div className="farm-desc-right">
            <FarmCard>
              <div className="container-box">
                <img src={plugin} alt="" />
                <div className="fully-desc">
                  <span className="sub-title">Fully Pluggable</span>
                  <span className="fully-content">
                    Everything inside Farm is powered by plugins, you can
                    achieve anything you want by writing a plugin. Support both
                    Rust and Js plugins.
                  </span>
                </div>
              </div>
            </FarmCard>
          </div>
        </div>
        <FarmCard>
          <div className="action-box">
            <div className="container-box">
              <img src={light} alt="" />
              <div className="fast-desc">
                <span className="sub-title">Super Fast </span>
                <span className="fast-content">
                  Farm's compiler is written in Rust, with multi-threading,
                  lazy/asynchronous compilation and persist caching, Farm can
                  start a project in milliseconds, perform a HMR update within
                  10ms.
                </span>
              </div>
            </div>
            <div className="action">
              <ButtonAction to="https://farmfe.org/docs/quick-start">
                Quick Start
              </ButtonAction>
              <ButtonAction to="https://farmfe.org/docs/why-farm">
                Why Farm ?
              </ButtonAction>
            </div>
          </div>
        </FarmCard>
      </div>
    </div>
  );
}
