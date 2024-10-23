export function safeJsonParse<T>(v: string, defaultValue?: T): T {
  try {
    return JSON.parse(v);
  } catch (error) {
    return defaultValue;
  }
}
