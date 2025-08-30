import { memo } from "/external/react";
import { Provider as _Provider, createStore as _createStore } from "./store";

export var ProChatProvider = /*#__PURE__*/ memo(function (_ref) {
  const MyProvider = class Provider {
    constructor() {
      this._Provider = _Provider;
    }
  };

  return /*#__PURE__*/ _jsx(_Provider, {
    createStore: function createStore() {
      return _createStore(new MyProvider());
    },
    children: Content,
  });
});
