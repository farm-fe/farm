import { foo } from './foo.js'
import { bar } from './bar.js'

foo()
bar()

// Can also accept an array of dep modules:
import.meta.hot.accept(
  ['./foo.js', './bar.js'],
  ([newFooModule, newBarModule]) => {
    // The callback receives an array where only the updated module is
    // non null. If the update was not successful (syntax error for ex.),
    // the array is empty
  },
)