import { Select as _Select } from './select';
import { SelectOption } from './select-option';

export type SelectComponentType = typeof _Select & {
  Option: typeof SelectOption;
};
(_Select as SelectComponentType).Option = SelectOption;

export const Select = _Select as SelectComponentType;

export type { SelectInstance } from './select';
