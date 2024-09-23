export const VITE_DEFAULT_ASSETS = [
  // images
  'apng',
  'png',
  'jpe?g',
  'jfif',
  'pjpeg',
  'pjp',
  'gif',
  'svg',
  'ico',
  'webp',
  'avif',

  // media
  'mp4',
  'webm',
  'ogg',
  'mp3',
  'wav',
  'flac',
  'aac',
  'opus',
  'mov',
  'm4a',
  'vtt',

  // fonts
  'woff2?',
  'eot',
  'ttf',
  'otf',

  // other
  'webmanifest',
  'pdf',
  'txt'
];
// the name of the virtual module internal the adapter
export const VITE_ADAPTER_VIRTUAL_MODULE = 'vite-adapter-virtual:';

export const VITE_EXTERNAL_KEYS: string[] = ['esbuild'];

export const RESERVED_OBJECT_PROPERTIES = [
  'then',
  'length',
  'constructor',
  'prototype'
];

export const EXTERNAL_KEYS = [
  ...VITE_EXTERNAL_KEYS,
  ...RESERVED_OBJECT_PROPERTIES
];
