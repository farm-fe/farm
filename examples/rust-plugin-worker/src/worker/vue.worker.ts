self.onmessage = (e) => {
  console.log("vue Message received from main script");
  const workerResult = `Result: ${e.data[0] * e.data[1]}`;
  console.log("Posting message back to main script");
  console.log("vue result:", workerResult);
  postMessage(workerResult);
};
