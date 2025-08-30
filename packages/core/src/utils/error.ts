export function convertErrorMessage(error: Error) {
  let errorMessages = [];

  try {
    const parsedErrors = JSON.parse(error.message);

    if (Array.isArray(parsedErrors)) {
      errorMessages = parsedErrors.map(parseErrorItem);
    } else {
      errorMessages = [parseErrorItem(parsedErrors)];
    }
  } catch {
    errorMessages = [error.message];
  }

  return errorMessages.join('\n\n');
}

function parseErrorItem(item: any): string {
  try {
    const parsedItem = typeof item === 'string' ? JSON.parse(item) : item;

    if (typeof parsedItem === 'object' && parsedItem !== null) {
      return parsedItem.message || JSON.stringify(parsedItem);
    } else {
      return String(parsedItem);
    }
  } catch {
    return String(item);
  }
}
