import { Checkbox as _Checkbox } from './checkbox';
import { CheckboxGroup } from './checkbox-group';

export type { CheckboxEvent } from './checkbox';

export type CheckboxComponentType = typeof _Checkbox & {
  Group: typeof CheckboxGroup;
};
(_Checkbox as CheckboxComponentType).Group = CheckboxGroup;

export const Checkbox = _Checkbox as CheckboxComponentType;
