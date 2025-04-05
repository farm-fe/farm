//index.js:
 const a = 'a-runtime.2.ts';
const b = 'b-runtime.2.ts';
function BB() {
    const a = 5;
    const b = 6;
    console.log(a, b);
}
const a$1 = 'a-runtime.1.ts';
const b$1 = 'b-runtime.1.ts';
function BB$1() {
    const a = 5;
    const b = 6;
    console.log(a, b);
}
console.log(a$1, b$1, a, b);
const a$2 = 'a-runtime.ts';
const b$2 = 'b-runtime.ts';
console.log(a$2, b$2, a$1, b$1);
