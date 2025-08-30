//index.js:
 import { memo } from "/external/react";
function createStore$1() {
    return "_createStore(props, devtoolOptions);";
}
class Provider$1 {
    constructor(props){
        this.props = props;
    }
}
var ProChatProvider = memo(function(_ref) {
    const MyProvider = class Provider {
        constructor(){
            this._Provider = Provider$1;
        }
    };
    return _jsx(Provider$1, {
        createStore: function createStore() {
            return createStore$1(new MyProvider());
        },
        children: Content
    });
});
console.log(ProChatProvider);
