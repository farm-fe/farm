import React from 'react';
import './index.css';

import logo from '../../../assets/logo.svg';

export function Title() {
  return (
    <div className="farm-title-wrapper">
      <img src={logo} className="farm-logo" alt="Farm Logo" />
      <h1 className="farm-title">Farm</h1>
    </div>
  );
}
