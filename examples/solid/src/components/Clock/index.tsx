import { type Component, createSignal, createEffect } from 'solid-js';

const Clock: Component = () => {
  const [now, setNow] = createSignal(new Date());

  createEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(timer);
  });

  return (
    <div>
      <p>It is {now().toLocaleTimeString()}.</p>
    </div>
  );
};

export default Clock;
