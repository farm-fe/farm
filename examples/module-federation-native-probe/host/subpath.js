// Known failing CJS-to-async dependency regression probe.
import {jsx} from 'react/jsx-runtime';
window.jsxProbe = jsx('span',{}).type;
