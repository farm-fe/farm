// const figlet = require('figlet')
// import figlet from 'figlet'
export const PKG = require('../../../package.json')
export const VITE_CLI_VERSION = PKG.version
export const JZZX_NAME = PKG.name
// export const VALUE_ONLINE =
//   '\r\n' +
//   figlet.textSync('VITE CLI', {
//     font: '3D-ASCII',
//     horizontalLayout: 'default',
//     verticalLayout: 'default',
//     width: 200,
//     whitespaceBreak: true
//   })

export const VERSION = `\n\t\t🌱🌱 Published${PKG.version}Build @ VITE-CLI.com 🌱🌱`
export const BUILD_DATE = `\n\t\t\t🌱🌱 Build last date: ${new Date().getFullYear()}-${
  new Date().getMonth() + 1
}-${new Date().getDate()} 🌱🌱`
