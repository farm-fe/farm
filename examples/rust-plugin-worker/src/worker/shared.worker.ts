self.onconnect = (event: MessageEvent) => {
  const ports = event.ports;
  if (!ports || ports.length === 0) {
    return;
  }

  const port = ports[0];
  port.onmessage = (messageEvent: MessageEvent<[number, number]>) => {
    const [a, b] = messageEvent.data;
    port.postMessage(`Shared result: ${a * b}`);
  };
};
