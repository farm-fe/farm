

export function nested() {
  const a = 100;
  if(!!a) {
    console.log(require('./cjs'));
  }
}