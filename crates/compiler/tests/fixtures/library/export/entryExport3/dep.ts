const a = 3;
const b = 4;
const c = 5;

function BB() {
  const a = 5;
  const b = 6;
  console.log(a, b);
}

export { a, b };

export default {
  a,
  b,
  c
};
