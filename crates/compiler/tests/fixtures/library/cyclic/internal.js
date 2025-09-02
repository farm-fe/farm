import statisticToken, { merge as mergeToken } from "./statistic";
import genComponentStyleHook from "./hook";

export { statisticToken, mergeToken, genComponentStyleHook };

export function useToken() {
  console.log("useToken");
}
