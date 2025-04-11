export const namedA = 10;
export const namedB = 20;
export const namedC = 30;

export { namedA as renamedA, namedB as renamedB, namedC as renamedC };

const sameNameWithFile_ts = 1;

const sameNameWithFile_ts_ns = 2;

export default {
  renamedA: namedA,
  renamedB: namedB,
  renamedC: namedC
};
