const value = await import('./data.js');

const dep2 = await new Promise((resolve) => {
  setTimeout(() => resolve(value.default), 200);
});

export default dep2;