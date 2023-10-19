const items = ['1', '2', '3', '4', '5'];

export default function () {
  return items.map((item) => {
    return `default-${item}`;
  });
}

export function named() {
  return items.map((item) => {
    return `named-${item}`;
  });
}
