export function handleErrorSync(
  fun: (...args: any[]) => void,
  args: any[],
  cb: (err: Error) => void = noop
) {
  try {
    fun(...args);
  } catch (err) {
    cb(err);
  }
}

// eslint-disable-next-line @typescript-eslint/no-empty-function
function noop() {}
