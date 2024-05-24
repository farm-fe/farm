export function convertErrorMessage(error: Error) {
  let errorMessage = '';

  try {
    errorMessage = JSON.parse(error.message).join('\n');
  } catch {
    errorMessage = error.message;
  }
  return errorMessage;
}
