import React from 'react';
import styles from './main.module.scss';
import './main.css';

export function Main() {
  return (
    <div className={'main'}>
      main<div className={`${styles.child} child-global`}>child</div>
    </div>
  );
}
