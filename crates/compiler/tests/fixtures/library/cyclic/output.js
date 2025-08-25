//index.js:
 const enableStatistic = process.env.NODE_ENV !== "production" || typeof CSSINJS_STATISTIC !== "undefined";
let recording = true;
function merge() {
    console.log(recording, enableStatistic);
}
function statisticToken(token) {
    console.log(token);
}
function genComponentStyleHook() {
    return (_prefixCls)=>{
        const prefixCls = _prefixCls.value;
        const [token, hashId] = useToken();
        return [
            useStyleRegister(componentInfo, ()=>{
                const { token: proxyToken } = statisticToken(token.value);
                const mergedToken = merge(proxyToken, {
                    prefixCls: prefixCls.value
                }, {});
                console.log(mergedToken);
            }),
            hashId
        ];
    };
}
function useToken() {
    console.log("useToken");
}
console.log(genComponentStyleHook);
