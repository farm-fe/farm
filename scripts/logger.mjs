export function logger(msg, { title = 'FARM INFO', color = 'green' } = {}) {
  const BASIC_COLORS = [
    'black',
    'red',
    'green',
    'yellow',
    'blue',
    'magenta',
    'cyan',
    'white'
  ];

  const EXTENDED_COLORS = {
    orange: 208 
  };

  const CUSTOM_COLORS = {
    rust: { r: 183, g: 65, b: 14 }
  };

  const COLOR_CODE = BASIC_COLORS.indexOf(color);
  const EXTENDED_COLOR_CODE = EXTENDED_COLORS[color];
  const CUSTOM_COLOR = CUSTOM_COLORS[color];

  let colorCodeStr = '';
  let titleStr = '';

  if (COLOR_CODE >= 0) {
    colorCodeStr = `\x1b[3${COLOR_CODE}m`;
    titleStr = title ? `\x1b[4${COLOR_CODE};30m ${title} \x1b[0m ` : '';
  } else if (EXTENDED_COLOR_CODE !== undefined) {
    colorCodeStr = `\x1b[38;5;${EXTENDED_COLOR_CODE}m`;
    titleStr = title ? `\x1b[48;5;${EXTENDED_COLOR_CODE}m\x1b[30m ${title} \x1b[0m ` : '';
  } else if (CUSTOM_COLOR) {
    const { r, g, b } = CUSTOM_COLOR;
    colorCodeStr = `\x1b[38;2;${r};${g};${b}m`;
    titleStr = title ? `\x1b[48;2;${r};${g};${b}m\x1b[30m ${title} \x1b[0m ` : '';
  } else {
    colorCodeStr = '';
    titleStr = title ? `${title} ` : '';
  }

  if (colorCodeStr) {
    console.log(`${titleStr}${colorCodeStr}${msg}\x1b[0m`);
  } else {
    console.log(`${titleStr}${msg}`);
  }
}
