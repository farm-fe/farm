//index.js:
 const a = 3;
class getA {
    constructor(){}
    getA() {
        return a;
    }
}
function getA$1() {
    return new getA();
}
const a$1 = 1;
console.log(a$1, getA$1);
