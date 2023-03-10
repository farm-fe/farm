import React, { useEffect, useState } from 'react';

import './index.css'

const COUNT_DOWN = 60;
const BUTTON_TEXT = BTN || 'Send';

export function CounterButton() {
  const [count, setCount] = useState(COUNT_DOWN);
  const [text, setText] = useState(BUTTON_TEXT);
  const [timer, setTimer] = useState(null as null | number);
  const [pause, setPause] = useState(false);

  useEffect(() => {
    return () => {
      if (timer) {
        clearInterval(timer);
      }
    }
  }, [timer]);

  const countdown = () => {
    console.log(timer, pause, count, text);
    setCount(count => {
      if (count == 0) {
        clearInterval(timer as number);
        setText(BUTTON_TEXT);
        return 0;
      }

      setText(`${count - 1}`);
      return count - 1;
    });
  };

  return <button className='counter-button' onClick={() => {
    if (timer && pause === false) {
      setPause(true);
      setText('Pause')

      clearInterval(timer);
      return;
    } else if (pause === true) {
      setPause(false);
      setText(`${count}`);

      setTimer(setInterval(countdown, 1000));
      return;
    }

    console.log(timer, pause, count, text);
    const t = setInterval(countdown, 1000);

    setText(`${COUNT_DOWN}`);
    setTimer(t);
  }}>{text}</button>
}
