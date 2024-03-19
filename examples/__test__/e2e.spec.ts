import { expect, test } from 'vitest'
import { browserErrors } from "~utils"

test("e2e tests for example", () => {
  expect(browserErrors).is.empty;
});
