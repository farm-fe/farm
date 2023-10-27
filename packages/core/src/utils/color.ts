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

const require = createRequire(import.meta.url);

const argv = process.argv || [],
  env = process.env;

const enabled =
  !('NO_COLOR' in env || argv.includes('--no-color')) &&
  ('FORCE_COLOR' in env ||
    argv.includes('--color') ||
    process.platform === 'win32' ||
    (require != null && require('tty').isatty(1) && env.TERM !== 'dumb') ||
    'CI' in env);

const formatter =
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

export const reset: any = enabled
  ? (s: string) => `\x1b[0m${s}\x1b[0m`
  : String;
export const bold: any = enabled
  ? formatter('\x1b[1m', '\x1b[22m', '\x1b[22m\x1b[1m')
  : String;
export const dim = enabled
  ? formatter('\x1b[2m', '\x1b[22m', '\x1b[22m\x1b[2m')
  : String;
export const italic: any = enabled ? formatter('\x1b[3m', '\x1b[23m') : String;
export const underline: any = enabled
  ? formatter('\x1b[4m', '\x1b[24m')
  : String;
export const inverse: any = enabled ? formatter('\x1b[7m', '\x1b[27m') : String;
export const hidden: any = enabled ? formatter('\x1b[8m', '\x1b[28m') : String;
export const strikethrough: any = enabled
  ? formatter('\x1b[9m', '\x1b[29m')
  : String;

export const debugColor = formatter('\x1b[38;2;255;140;0m', '\x1b[39m');
export const brandColor = enabled
  ? formatter('\x1b[38;2;113;26;95m', '\x1b[39m')
  : String;
export const black = enabled ? formatter('\x1b[30m', '\x1b[39m') : String;
export const red = enabled ? formatter('\x1b[31m', '\x1b[39m') : String;
export const green = enabled ? formatter('\x1b[32m', '\x1b[39m') : String;
export const yellow = enabled ? formatter('\x1b[33m', '\x1b[39m') : String;
export const blue = enabled ? formatter('\x1b[34m', '\x1b[39m') : String;
export const magenta = enabled ? formatter('\x1b[35m', '\x1b[39m') : String;
export const purple = enabled
  ? formatter('\x1b[38;2;173;127;168m', '\x1b[39m')
  : String;
export const cyan = enabled ? formatter('\x1b[36m', '\x1b[39m') : String;
export const white = enabled ? formatter('\x1b[37m', '\x1b[39m') : String;
export const gray = enabled ? formatter('\x1b[90m', '\x1b[39m') : String;
export const bgBlack = enabled ? formatter('\x1b[40m', '\x1b[49m') : String;
export const bgRed = enabled ? formatter('\x1b[41m', '\x1b[49m') : String;
export const bgGreen = enabled ? formatter('\x1b[42m', '\x1b[49m') : String;
export const bgYellow = enabled ? formatter('\x1b[43m', '\x1b[49m') : String;
export const bgBlue = enabled ? formatter('\x1b[44m', '\x1b[49m') : String;
export const bgMagenta = enabled ? formatter('\x1b[45m', '\x1b[49m') : String;
export const bgCyan = enabled ? formatter('\x1b[46m', '\x1b[49m') : String;
export const bgWhite = enabled ? formatter('\x1b[47m', '\x1b[49m') : String;
