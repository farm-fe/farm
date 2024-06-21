export const isString = (v: any): v is string => {
  console.log('isString2');
  return typeof v === 'string'
};
