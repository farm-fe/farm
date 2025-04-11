//index.js:
 const hello$2 = 'hello1';
const hello$1 = 'hello';
console.log(hello$2, hello$1);
function export_nested() {
    const hello = 'hello';
    console.log(hello);
}
var hello = hello$2;
function say() {
    var hello = hello$2;
    var hello$1 = hello$2;
    console.log(hello);
    function nested_say() {
        var hello = hello$2;
        var hello$2 = hello$2;
        console.log(hello);
    }
}
say();
