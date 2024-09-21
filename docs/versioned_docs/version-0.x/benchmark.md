# Benchmarks

## Introduction

Using Turbopack's bench cases (1000 React components), see https://turbo.build/pack/docs/benchmarks.

### Run this benchmark yourself

> Test Repo：https://github.com/farm-fe/performance-compare
>
> Test Machine（Linux Mint 21.1 Cinnamon， 11th Gen Intel© Core™ i5-11400 @ 2.60GHz × 6， 15.5 GiB）

```ts
# Install dependencies
pnpm install

# run benchmark
pnpm benchmark
```

### Data

|         | **Startup** | **HMR (Root)** | **HMR (Leaf)** | **Production Build** |
| ------- | ----------- | -------------- | -------------- | -------------------- |
| Webpack | 8035ms      | 345ms          | 265ms          | 11321ms              |
| Vite    | 3078ms      | 35ms           | 18ms           | 2266ms               |
| Rspack  | 831ms       | 104ms          | 96ms           | 724ms                |
| Farm    | 403ms       | 11ms           | 10ms           | 288ms                |

---

## metrics

- Cold StartUp Time: The time it takes to develop a build without caching

- Hot StartUp Time: The time it takes to develop a build with caching

- Cold Production Build Time: The time it takes to build a production build without caching

- Hot Production Build Time: The time it takes to build a production build with caching

- HMR Time: The time it takes to apply an update to a file and send it to the development server to the response

  - HMR Root: The time for updating a react component file that has no dependency

  - HMR Leaf: The time for updating a root react component, normally it is named `App.tsx` or `index.tsx`

### Benchmark for all metrics

<!-- ![performance](/img/20231204223204.png) -->

<img style={{width: '100%',borderRadius: '8px', border: '2px solid #8f1a7f60'}} src="/img/20231204223204.png" />

### Benchmark of HMR

<!-- ![performance](/img/hmr-linux.png) -->

<img style={{width: '100%',borderRadius: '8px', border: '2px solid #8f1a7f60'}} src="/img/hmr-linux.png" />

### Benchmark of Startup

<!-- ![performance](/img/startup-linux.png) -->

<img style={{width: '100%',borderRadius: '8px', border: '2px solid #8f1a7f60'}} src="/img/startup-linux.png" />

### Benchmark of Production Build

<!-- ![performance](/img/build-linux.png) -->

<img style={{width: '100%',borderRadius: '8px', border: '2px solid #8f1a7f60'}} src="/img/build-linux.png" />
