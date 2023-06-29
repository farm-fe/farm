import React from 'react';

import styles from './main.module.scss';

export function Child() {
  return <div className={`${styles.child} child-global`}>child</div>;
}
