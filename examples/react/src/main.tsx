import React from 'react';
import { useStore } from 'react-redux';

import { Welcome } from './components/index';
import './main.css';

import { BizType } from './enums';

import * as Sentry from '@sentry/react';
import { Effect } from 'effect';

Sentry.init({});

const result = Effect.runSync(Effect.succeed(42));

export function Main() {
  const store = useStore();
  console.log(import.meta.env);
  return (
    <>
      <div style={{ color: '#fff' }}>
        <div>effect: {result}</div>
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
