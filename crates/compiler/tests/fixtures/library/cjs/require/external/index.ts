import './dep.cjs'

export const loaders = {
  '.js': require,
  '.cjs': require,
  '.json': require,
}

export default 'require-external'