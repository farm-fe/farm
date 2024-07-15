import React from 'react';
import { useStore } from 'react-redux';

import { Welcome } from './components/index';
import './main.css';

import { BizType } from './enums';

export function Main() {
  const store = useStore();
  console.log(import.meta.env);
  return (
    <>
      <div>
        <div style={{ width: '100px', color: '#fff' }}>
          <b>store.api.config.online: </b>
          {JSON.stringify(store.getState().api.config.online)}
          BizType: {BizType.First} {BizType.Second}
        </div>
      </div>
      <Welcome />
    </>
  );
}
