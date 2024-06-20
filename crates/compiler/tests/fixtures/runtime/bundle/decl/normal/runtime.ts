import "./runtime.1";

const a = 1;
const b = 2;
console.log(a, b);

{
  const a = 1;
  const b = 2;
}

for (var for1 in [1, 2, 3]) {
  console.log(for1);
}

// @ts-ignore
for (var for1 of [1, 2, 3]) {
  console.log(for1);
}

for (var for2 of [1, 2, 3]) {
  console.log(for2);
}

for (var for3 = 123; for3 < 234; for3++) {
  console.log(for3);
}

for (const for3 = 123; for3 < 234; for3) {
  break;
}
