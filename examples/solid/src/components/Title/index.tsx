import { type Component } from 'solid-js';

import logo from '../../../assets/logo.svg';
import './index.css';

export const Title: Component = () => {
  return (
    <div class="farm-title-wrapper">
      <img src={logo} class="farm-logo" alt="Farm Logo" />
      <h1 class="farm-title">Farm</h1>
    </div>
  );
};
