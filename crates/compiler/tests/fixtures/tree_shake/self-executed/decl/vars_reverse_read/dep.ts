const a1 = {};

const b1 = { a1 };

export { a1 };

const a2 = {};
const b2 = { a2 };

b2.a2.aaa = 2;

export { a2 };
