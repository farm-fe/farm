export function logger(msg, { title = 'FARM INFO', color = 'green' } = {}) {
  const COLOR_CODE = [
    'black',
    'red',
    'green',
    'yellow',
    'blue',
    'magenta',
    'cyan',
    'white'
  ].indexOf(color);
  if (COLOR_CODE >= 0) {
    const TITLE_STR = title ? `\x1b[4${COLOR_CODE};30m ${title} \x1b[0m ` : '';
    console.log(`${TITLE_STR}\x1b[3${COLOR_CODE}m${msg}\x1b[;0m`);
  } else {
    console.log(title ? `${title} ${msg}` : msg);
  }
}
