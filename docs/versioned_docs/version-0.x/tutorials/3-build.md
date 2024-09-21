# Build For Production
By default, Farm has enabled support for the following features for production builds:
* **`Tree Shake`**: Crop and filter irrelevant modules and code
* **`Compression`**: Compress and mix the product to reduce the product volume
* **`Automatically inject Polyfill`**: Farm downgrades projects to ES5 by default, which means that the products built by Farm can run on almost all browsers
* **`Automatic partial packaging`**: Based on dependencies and size, the project is partially packaged. For each resource request, about 25 resources are generated to ensure parallel loading performance and improve cache hits as much as possible. Rate

## Add build script
Add build script in `package.json`:
```json title="package.json" {7}
{
   "name": "1-create-a-project",
   "version": "1.0.0",
   "scripts": {
     "test": "echo \"Error: no test specified\" && exit 1",
     "start": "farm start",
     "build": "farm build"
   },
   // ...ignore other fields
}
```
Then execute `npm run build`.

## Configure Tree Shake and compression
* [Tree Shake](/docs/features/tree-shake)
* [Minification](/docs/features/minification)

## Configure Partial Bundling
* [Partial Bundling](/docs/features/partial-bundling)
