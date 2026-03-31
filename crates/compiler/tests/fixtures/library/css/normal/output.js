//index.css:
 .container {
  color: red;
  font-size: 16px;
}

//index.js:
 import "style.css";
function hello() {
    return 'hello';
}
export { hello as hello };
