import React from 'react';
import { Welcome } from './components/index';
import './main.css';
import { a } from './test';

declare const BTN: string;
declare const IRRELEVANT_ESCAPE_ENV: string;
export function Main() {
  console.log(a);

  return (
    <>
      <Welcome />
    </>
  );
}
