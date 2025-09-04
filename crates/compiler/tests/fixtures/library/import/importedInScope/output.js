//index.js:
 const hello$4 = 'hello1';
const hello$1$1 = 'hello';
console.log(hello$4, hello$1$1);
function export_nested() {
    const hello = 'hello';
    console.log(hello);
}
var hello$3 = hello$4;
function say() {
    var hello = hello$4;
    var hello$1 = hello$4;
    console.log(hello);
    function nested_say() {
        var hello = hello$4;
        var hello$2 = hello$4;
        console.log(hello);
    }
}
say();
