import { nativeWindow, emptyWindow } from "./dep3"; // should be preserved

export const a: any = {
  field: '1',
  Camera: undefined,
}; // should be preserved

const nativeLocation = nativeWindow.location; // should be preserved

export const b: any = nativeLocation; // should be preserved cause it may contain side effects

export const c: any = {}; // should be removed as it is not used

export const d = emptyWindow; // should be removed as it is not used and does not contain side effects