//index.js:
 const isString = (v)=>{
    console.log('isString1');
    return typeof v === 'string';
};
const isString$1 = (v)=>{
    console.log('isString2');
    return typeof v === 'string';
};
const { document, addEventListener, removeEventListener } = window;
export { addEventListener as addEventListener, isString as isString1, removeEventListener as removeEventListener, isString$1 as isString2, document as document };
