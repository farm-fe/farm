import React from 'react';

export interface SelectContext {
  value?: string | string[];
  updateValue?: (next: string) => unknown;
  visible?: boolean;
  updateVisible?: (next: boolean) => unknown;
  disableAll: boolean;
  ref: React.RefObject<HTMLDivElement>;
}

const defaultContext = <SelectContext>{
  visible: false,
  disableAll: false
};

const selectContext = React.createContext<SelectContext>(defaultContext);

export function useSelect() {
  return React.useContext(selectContext);
}

export const { Provider } = selectContext;
