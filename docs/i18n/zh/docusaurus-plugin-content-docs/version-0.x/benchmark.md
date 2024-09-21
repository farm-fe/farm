# Benchmarks

Using Turbopack's bench cases (1000 React components), see https://turbo.build/pack/docs/benchmarks.

> Test Repo：https://github.com/farm-fe/performance-compare
>
> Test Machine（Linux Mint 21.1 Cinnamon， 11th Gen Intel© Core™ i5-11400 @ 2.60GHz × 6， 15.5 GiB）

![xx](/img/benchmark.png)

|                     | **Startup**  | **HMR (Root)**  | **HMR (Leaf)**  |
| ------------------- | ------- | ----- | --- |
| Webpack      | 7694ms   | 334ms | 267ms |
| Vite         | 4625ms  | 32ms  | 27ms |
| Turbopack   | 2444ms | 9ms | 11ms |
| Rspack   | 406ms | 311ms | 301ms |
| Farm    | 395ms  | 7ms  | 12ms  |

