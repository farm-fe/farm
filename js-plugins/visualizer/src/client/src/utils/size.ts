export function formatSize(bytes?: number) {
  if (bytes === undefined || bytes === null) {
    return "0 bytes";
  }
  let size: string;

  if (bytes >= 1073741824) {
    size = `${(bytes / 1073741824).toFixed(2)} GB`;
  } else if (bytes >= 1048576) {
    size = `${(bytes / 1048576).toFixed(2)} MB`;
  } else if (bytes >= 1024) {
    size = `${(bytes / 1024).toFixed(2)} KB`;
  } else if (bytes > 1) {
    size = `${bytes} bytes`;
  } else if (bytes === 1) {
    size = `${bytes} byte`;
  } else {
    size = "0 bytes";
  }
  return size;
}
