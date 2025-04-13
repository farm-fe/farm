import { ReactReduxContext } from './Context';
import { useReduxContext as useDefaultReduxContext } from './useReduxContext';

import './foo';

export function createSelectorHook(context) {
  if (context === void 0) {
    context = ReactReduxContext;
  }

  var useReduxContext = context === ReactReduxContext ? useDefaultReduxContext : function () {
    return context;
  };

  return function useSelector() {
    var _useReduxContext = useReduxContext();
    return _useReduxContext
  };
}