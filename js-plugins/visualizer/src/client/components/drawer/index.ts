import { DrawerContent } from './content';
import { Drawer as _Drawer } from './drawer';

export type DrawerComponentType = typeof _Drawer & {
  Content: typeof DrawerContent;
};
(_Drawer as DrawerComponentType).Content = DrawerContent;

export const Drawer = _Drawer as DrawerComponentType;
