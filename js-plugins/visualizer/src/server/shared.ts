import fs from 'node:fs';
import type { uint8 } from './interface';

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export function stringToByte(b: string | ArrayLike<uint8>) {
  if (typeof b === 'string') {
    return encoder.encode(b);
  }
  return new Uint8Array(b);
}

export function byteToString(b: string | ArrayLike<uint8>) {
  if (typeof b === 'string') {
    return b;
  }
  return decoder.decode(new Uint8Array(b));
}

// MIT License
// Copyright (c) Vite

export function tryStatSync(file: string): fs.Stats | undefined {
  try {
    // The "throwIfNoEntry" is a performance optimization for cases where the file does not exist
    return fs.statSync(file, { throwIfNoEntry: false });
  } catch {
    // Ignore errors
  }
}

export function isFileReadable(filename: string): boolean {
  if (!tryStatSync(filename)) {
    return false;
  }

  try {
    // Check if current process has read permission to the file
    fs.accessSync(filename, fs.constants.R_OK);

    return true;
  } catch {
    return false;
  }
}

export function slash(path: string) {
  const isExtendedLengthPath = /^\\\\\?\\/.test(path);
  if (isExtendedLengthPath) {
    return path;
  }
  return path.replace(/\\/g, '/');
}
