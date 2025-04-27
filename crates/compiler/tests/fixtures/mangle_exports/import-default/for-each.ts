function forEach<T>(arr: T[], fn: (item: T) => void) {
  for (let i = 0; i < arr.length; i++) {
    fn(arr[i]);
  }
}

export default forEach;