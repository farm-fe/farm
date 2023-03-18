export function handleErrorSync(
  fun: Function,
  args: any[],
  cb: Function = noop
) {
  try {
    fun(...args);
  } catch (err) {
    cb(err);
  }
}

function noop() {}
