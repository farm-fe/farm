//index.js:
 const ReactReduxContext = {};
function useReduxContext$1() {
    var contextValue = ReactReduxContext;
    if (process.env.NODE_ENV !== 'production' && !contextValue) {
        throw new Error('could not find react-redux context value; please ensure the component is wrapped in a <Provider>');
    }
    return contextValue;
}
console.log(useReduxContext$1());
function createSelectorHook(context) {
    if (context === void 0) {
        context = ReactReduxContext;
    }
    var useReduxContext = context === ReactReduxContext ? useReduxContext$1 : function() {
        return context;
    };
    return function useSelector() {
        var _useReduxContext = useReduxContext();
        return _useReduxContext;
    };
}
export { createSelectorHook as createSelectorHook };
