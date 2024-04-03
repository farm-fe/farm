const a1 = {};

const b1 = { a1 };

const c1 = { b1 };

const d1 = { c1 };

const e1 = { d1 };

export { a1 };

const a2 = {};
const b2 = { a2 };

b2.a2.aaa = 2;

export { a2 };

const a3 = {};

const b3 = { a3 };

console.log(b3);

const c3 = { b3 };

console.log(c3);

const d3 = { c3 };

const e3 = { d3 };

export { a3 };
