

const path1 = 'foo';
const bar = 'bar';

new URL(`./foo/${path1}/${bar}`, import.meta.url)

new URL(`./foo/${path1}-${bar}`, import.meta.url)

new URL(`./foo/${path1}/**/${bar}`, import.meta.url)

new URL("./foo/**/*/**", import.meta.url)