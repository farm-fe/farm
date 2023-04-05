import { type Component } from 'solid-js';
import { Title } from './components/Title';
import { Description } from './components/Description';

import './App.css';

const App: Component = () => {
  return (
    <>
      <Title />
      <Description />
    </>
  );
};

export default App;
