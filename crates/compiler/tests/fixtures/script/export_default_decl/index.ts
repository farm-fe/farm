
export default function myPlugin(options = {}) {
  console.log('myPlugin', options);
  return {
    visitor: {}
  };
}