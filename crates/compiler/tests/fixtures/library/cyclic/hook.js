import { mergeToken, statisticToken, useToken } from "./internal";

export default function genComponentStyleHook() {
  return (_prefixCls) => {
    const prefixCls = _prefixCls.value;
    const [token, hashId] = useToken();

    return [
      useStyleRegister(componentInfo, () => {
        const { token: proxyToken } = statisticToken(token.value);
        const mergedToken = mergeToken(
          proxyToken,
          {
            prefixCls: prefixCls.value,
          },
          {},
        );

        console.log(mergedToken);
      }),
      hashId,
    ];
  };
}
