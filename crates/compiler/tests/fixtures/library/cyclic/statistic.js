const enableStatistic =
  process.env.NODE_ENV !== "production" ||
  typeof CSSINJS_STATISTIC !== "undefined";
let recording = true;

export function merge() {
  console.log(recording, enableStatistic);
}

export default function statisticToken(token) {
  console.log(token);
}
