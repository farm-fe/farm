// import {stringifyPosition} from 'unist-util-stringify-position';

// const node = {
//   type: 'text',
//   value: 'Hello, world!',
//   position: {
//     start: { line: 1, column: 1, offset: 0 },
//     end: { line: 1, column: 13, offset: 12 }
//   }
// };

// const positionString = stringifyPosition(node);
// console.log(positionString); 
import {codes, constants, types, values} from 'micromark-util-symbol'

console.log(codes.atSign) // 64
console.log(constants.characterReferenceNamedSizeMax) // 31
console.log(types.definitionDestinationRaw) // 'definitionDestinationRaw'
console.log(values.atSign) // '@'