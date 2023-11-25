import React from 'react';
import { Welcome } from './components/index';
import './main.css';

// import { Button } from '@farmfe-examples/lib-for-browser';

console.log(process.env.FARM_BASE_TEST);

export function Main() {
  return (
    <>
      {/* <Button /> */}
      <Welcome />
    </>
  );
}
