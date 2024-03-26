module.exports = {
  plugins: {
    'postcss-pxtorem': {
      rootValue: 16,
      propList: ['*']
    },
    'tailwindcss/nesting': {},
    tailwindcss: {}
  }
};