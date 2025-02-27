import { noop } from 'foxact/noop';
import { createContext, useContext } from 'react';

export interface CheckboxContext {
  disabledAll: boolean;
  values: string[];
  inGroup: boolean;
  updateState: (val: string, checked: boolean) => void;
}

const initialValue: CheckboxContext = {
  disabledAll: false,
  values: [],
  inGroup: false,
  updateState: noop
};

export const CheckboxContext = createContext<CheckboxContext>(initialValue);

export function useCheckbox() {
  return useContext(CheckboxContext);
}

export const CheckboxProvider = CheckboxContext.Provider;
