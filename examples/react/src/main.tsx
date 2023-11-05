import React from 'react';
import { Welcome } from './components/index';
import './main.css';

declare const BTN: string;
declare const IRRELEVANT_ESCAPE_ENV: string;
console.log(process.env.FARM_BASE_TEST);

export function Main() {
  return (
    <>
      <Welcome />
    </>
  );
}
