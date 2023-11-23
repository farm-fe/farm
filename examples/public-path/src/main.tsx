import React from 'react';
import { Welcome } from './components/index';
import './main.css';

console.log(process.env.FARM_BASE_TEST);

export function Main() {
  return (
    <>
      <Welcome />
    </>
  );
}
