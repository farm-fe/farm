import ReactDom from 'react-dom';
import React from 'react';
import $ from 'jquery';

document.body.innerHTML = `
<div id="root">
  <div>jquery: ${$()}</div>
  <div>react-dom: ${ReactDom}</div>
  <div>react: ${React}</div>
</div>
`;
