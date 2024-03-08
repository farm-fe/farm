import React from 'react';
import { useStore } from 'react-redux';

import { Welcome } from './components/index';
import './main.css';
import { Button } from '@farmfe-examples/lib-for-browser';

export function Main() {
  const store = useStore();

  return (
    <>
      <div>
        <div style={{ width: '100px', color: '#fff' }}>
          <b>store.api.config.online: </b>
          {JSON.stringify(store.getState().api.config.online)}
        </div>
        <Button />
      </div>
      <Welcome />
    </>
  );
}
