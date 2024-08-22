module.exports = {
  plugins: [
    require('postcss-pxtorem')({
      rootValue: 16,
      propList: ['*'],
    }),
    require('@unocss/postcss')
  ]
}