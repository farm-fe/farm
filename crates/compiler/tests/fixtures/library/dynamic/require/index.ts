import { loadTsSync, loadTs } from "./loader";

console.log(loadTsSync());
console.log(await loadTs());