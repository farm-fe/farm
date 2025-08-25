const d = await import("./dynamic-entry");
console.log('ddd', d, d?.default);
export default d?.default;
