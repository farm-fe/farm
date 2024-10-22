# Benchmarks

使用 Turbopack 的基准案例（1000 个 React 组件），请参阅 https://turbo.build/pack/docs/benchmarks.

> 测试仓库：https://github.com/farm-fe/performance-compare
>
> 测试机器：(Linux Mint 21.1 Cinnamon， 11th Gen Intel© Core™ i5-11400 @ 2.60GHz × 6， 15.5 GiB)

![xx](/img/benchmark.png)

|                     | **Startup**  | **HMR (Root)**  | **HMR (Leaf)**  |
| ------------------- | ------- | ----- | --- |
| Webpack      | 7694ms   | 334ms | 267ms |
| Vite         | 4625ms  | 32ms  | 27ms |
| Turbopack   | 2444ms | 9ms | 11ms |
| Rspack   | 406ms | 311ms | 301ms |
| Farm    | 395ms  | 7ms  | 12ms  |

