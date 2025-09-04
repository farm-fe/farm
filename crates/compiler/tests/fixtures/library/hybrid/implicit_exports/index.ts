export function createResult() {
  return {
    field: 1,
    exports // implicit using global exports, farm will treat this module as a hybrid module
  }
}