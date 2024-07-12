
const foo = 'bar';

console.log(new URL(`./foo/${foo}.txt`, import.meta.url))

console.log(new URL(`./foo/${foo}`, import.meta.url))