// eslint-disable-next-line @typescript-eslint/no-unused-vars
import React, { createContext } from 'react';

export interface TabData {
  [key: string]: number | undefined;
}

export interface ITabDataContext {
  tabData: TabData;
  setTabData: (data: TabData) => void;
}

export const TabDataContext = createContext<ITabDataContext>({
  tabData: {},
  setTabData: () => {},
});
