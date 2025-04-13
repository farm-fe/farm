// ISC License

// Copyright (c) 2021 Alexey Raspopov, Kostiantyn Denysov, Anton Verinov

// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.

// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

import { createRequire } from 'node:module';

export type StylerEnabled = (s: any) => any;
export type ColorFunction = (s: any) => any;

const require = createRequire(import.meta.url);

// brand gradient colors
const gradientPurpleColor = [176, 106, 179];
const gradientPinkColor = [198, 66, 110];
const brandGradientColors = [255, 182, 193];
const brandGradientColors2 = [128, 0, 128];
const gradientOrangeColor = [255, 165, 0];
const gradientGoldColor = [255, 215, 0];

const argv = process.argv || [],
  env = process.env;

const enabled =
  !('NO_COLOR' in env || argv.includes('--no-color')) &&
  ('FORCE_COLOR' in env ||
    argv.includes('--color') ||
    process.platform === 'win32' ||
    (require != null && require('tty').isatty(1) && env.TERM !== 'dumb') ||
    'CI' in env);

const createFormatter =
  (open: string, close: string, replace = open) =>
  (input: string) => {
    const string = '' + input;
    const index = string.indexOf(close, open.length);
    return ~index
      ? open + replaceClose(string, close, replace, index) + close
      : open + string + close;
  };

const replaceClose = (
  string: string,
  close: string,
  replace: string,
  index: number
): string => {
  const start = string.substring(0, index) + replace;
  const end = string.substring(index + close.length);
  const nextIndex = end.indexOf(close);
  return ~nextIndex
    ? start + replaceClose(end, close, replace, nextIndex)
    : start + end;
};

export const reset: StylerEnabled = enabled
  ? (s: string) => `\x1b[0m${s}\x1b[0m`
  : String;
export const bold: StylerEnabled = enabled
  ? createFormatter('\x1b[1m', '\x1b[22m', '\x1b[22m\x1b[1m')
  : String;
export const dim = enabled
  ? createFormatter('\x1b[2m', '\x1b[22m', '\x1b[22m\x1b[2m')
  : String;
export const italic: StylerEnabled = enabled
  ? createFormatter('\x1b[3m', '\x1b[23m')
  : String;
export const underline: StylerEnabled = enabled
  ? createFormatter('\x1b[4m', '\x1b[24m')
  : String;
export const inverse: StylerEnabled = enabled
  ? createFormatter('\x1b[7m', '\x1b[27m')
  : String;
export const hidden: StylerEnabled = enabled
  ? createFormatter('\x1b[8m', '\x1b[28m')
  : String;
export const strikethrough: StylerEnabled = enabled
  ? createFormatter('\x1b[9m', '\x1b[29m')
  : String;

export const debugColor = createFormatter('\x1b[38;2;255;140;0m', '\x1b[39m');
export const brandColor = enabled
  ? createFormatter('\x1b[38;2;113;26;95m', '\x1b[39m')
  : String;

// black
export const black = enabled
  ? createFormatter('\x1b[38;2;0;0;0m', '\x1b[39m')
  : String;
export const red = enabled
  ? createFormatter('\x1b[38;2;219;90;107m', '\x1b[39m')
  : String;
export const green = enabled ? createFormatter('\x1b[32m', '\x1b[39m') : String;
export const yellow = enabled
  ? createFormatter('\x1b[33m', '\x1b[39m')
  : String;
export const blue = enabled
  ? createFormatter('\x1b[38;2;68;206;246m', '\x1b[39m')
  : String;
export const magenta = enabled
  ? createFormatter('\x1b[38;2;180;0;100m', '\x1b[39m')
  : String;
export const purple = enabled
  ? createFormatter('\x1b[38;2;140;67;86m', '\x1b[39m')
  : String;
export const orange = enabled
  ? createFormatter('\x1b[38;2;255;137;54m', '\x1b[39m')
  : String;
export const lightCyan = enabled
  ? createFormatter('\x1b[38;2;180;240;240m', '\x1b[39m')
  : String;
export const cyan = enabled ? createFormatter('\x1b[36m', '\x1b[39m') : String;
export const white = enabled ? createFormatter('\x1b[37m', '\x1b[39m') : String;

export const bgBlack = enabled
  ? createFormatter('\x1b[40m', '\x1b[49m')
  : String;

export const bgRed = enabled ? createFormatter('\x1b[41m', '\x1b[49m') : String;
export const bgGreen = enabled
  ? createFormatter('\x1b[42m', '\x1b[49m')
  : String;
export const bgYellow = enabled
  ? createFormatter('\x1b[43m', '\x1b[49m')
  : String;
export const bgBlue = enabled
  ? createFormatter('\x1b[44m', '\x1b[49m')
  : String;
export const bgMagenta = enabled
  ? createFormatter('\x1b[45m', '\x1b[49m')
  : String;
export const bgCyan = enabled
  ? createFormatter('\x1b[46m', '\x1b[49m')
  : String;
export const bgWhite = enabled
  ? createFormatter('\x1b[47m', '\x1b[49m')
  : String;

export function gradientString(text: string, colors: number[][]) {
  const steps = text.length;
  const gradient = colors.map(
    (color: number[]) => `\x1b[38;2;${color[0]};${color[1]};${color[2]}m`
  );

  let output = '';

  for (let i = 0; i < steps; i++) {
    const colorIndex = Math.floor((i / steps) * (colors.length - 1));
    output += `${gradient[colorIndex]}${text[i]}`;
  }

  output += '\x1b[0m';

  return output;
}

export function interpolateColor(
  color1: number[],
  color2: number[],
  factor: number
) {
  return [
    Math.round(color1[0] + (color2[0] - color1[0]) * factor),
    Math.round(color1[1] + (color2[1] - color1[1]) * factor),
    Math.round(color1[2] + (color2[2] - color1[2]) * factor)
  ];
}

export const PersistentCacheBrand =
  brandColor('⚡️') +
  gradientString(`FULL EXTREME!`, [
    gradientPurpleColor,
    interpolateColor(gradientPurpleColor, gradientPinkColor, 0.2),
    interpolateColor(gradientPurpleColor, gradientPinkColor, 0.4),
    interpolateColor(gradientPurpleColor, gradientPinkColor, 0.6),
    interpolateColor(gradientPurpleColor, gradientPinkColor, 0.8),
    gradientPinkColor,
    interpolateColor(gradientPinkColor, gradientOrangeColor, 0.3),
    interpolateColor(gradientPinkColor, gradientOrangeColor, 0.6),
    gradientOrangeColor,
    interpolateColor(gradientOrangeColor, gradientGoldColor, 0.5),
    gradientGoldColor
  ]);

export function handleBrandText(text: string) {
  console.log(
    gradientString(text, [
      brandGradientColors,
      interpolateColor(brandGradientColors, brandGradientColors2, 0.2),
      interpolateColor(brandGradientColors, brandGradientColors2, 0.4),
      interpolateColor(brandGradientColors, brandGradientColors2, 0.6),
      interpolateColor(brandGradientColors, brandGradientColors2, 0.8),
      brandGradientColors2
    ])
  );
}

export const BrandText = (text: string) =>
  gradientString(`\n${text} \n`, [
    brandGradientColors,
    interpolateColor(brandGradientColors, brandGradientColors2, 0.2),
    interpolateColor(brandGradientColors, brandGradientColors2, 0.4),
    interpolateColor(brandGradientColors, brandGradientColors2, 0.6),
    interpolateColor(brandGradientColors, brandGradientColors2, 0.8),
    brandGradientColors2
  ]);

export const colors = {
  reset,
  bold,
  dim,
  italic,
  underline,
  inverse,
  hidden,
  strikethrough,
  black,
  red,
  green,
  yellow,
  blue,
  magenta,
  purple,
  orange,
  cyan,
  white,
  bgBlack,
  bgRed,
  bgGreen,
  bgYellow,
  bgBlue,
  bgMagenta,
  bgCyan,
  bgWhite,
  debugColor,
  brandColor,
  handleBrandText,
  BrandText
};
