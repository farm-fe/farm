export const isString = (v: any): v is string => {
  console.log('isString1');
  return typeof v === 'string'
};
