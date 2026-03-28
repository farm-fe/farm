//greeting-ddf39ce6.js:
 function greet(name) {
    return `Hello, ${name}!`;
}
var greeting_ts_namespace_farm_internal_ = {
    greet: greet,
    __esModule: true
};
export { greet as greet };


//index.js:
 function add(a, b) {
    return a + b;
}
function multiply(a, b) {
    return a * b;
}
function main() {
    return add(1, 2) + ' ' + multiply(3, 4);
}
const loadGreeting = ()=>import('./greeting');
export { add as add, loadGreeting as loadGreeting, main as main, multiply as multiply };
