//index.js:
 const a = 3;
const b = 4;
function BB() {
    const a = 5;
    const b = 6;
    console.log(a, b);
}
const a$1 = 3;
const b$1 = 4;
function BB$1() {
    const a = 5;
    const b = 6;
    console.log(a, b);
}
console.log(a$1, b$1);
const for1 = 'for1';
const for2 = 'for2';
const for3 = 'for3';
const a$2 = 1;
const b$2 = 2;
console.log(a$2, b$2);
{
    const a = 1;
    const b = 2;
}for(var for1$2 in [
    1,
    2,
    3
]){
    console.log(for1$2);
}
for (var for1$2 of [
    1,
    2,
    3
]){
    console.log(for1$2);
}
for (var for2$1 of [
    1,
    2,
    3
]){
    console.log(for2$1);
}
for(var for3$1 = 123; for3$1 < 234; for3$1++){
    console.log(for3$1);
}
for(const for3$2 = 123; for3$2 < 234; for3$2){
    break;
}
