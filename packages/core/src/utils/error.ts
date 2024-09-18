export function convertErrorMessage(error: Error) {
  let errorMessages = [];

  try {
    const parsedErrors = JSON.parse(error.message);
    if (Array.isArray(parsedErrors)) {
      errorMessages = parsedErrors.map(
        (item) => JSON.parse(item).message || String(item)
      );
    } else {
      errorMessages = [
        JSON.parse(parsedErrors).message || String(parsedErrors)
      ];
    }
  } catch {
    errorMessages = [error.message];
  }

  return errorMessages.join('\n');
}
