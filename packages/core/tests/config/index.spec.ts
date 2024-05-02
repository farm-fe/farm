import path from "path";
import { describe, expect, test } from "vitest";
import { mergeFarmCliConfig } from "../../src/config/mergeConfig.js";

describe("mergeFarmCliConfig", () => {
  test("inlineOption.root not empty", () => {
    const result = mergeFarmCliConfig({}, { root: "/path/to/" });

    expect(result).toEqual({ root: "/path/to/" });
  });

  test("userConfig.root not empty", () => {
    const result = mergeFarmCliConfig({ root: "/path/to/" }, {});

    expect(result).toEqual({ root: "/path/to/" });
  });

  test("userConfig.root both inlineOption not empty", () => {
    const result = mergeFarmCliConfig(
      { root: "/path/to/inlineOption" },
      { root: "/path/to/userConfig" }
    );

    expect(result).toEqual({ root: "/path/to/userConfig" });
  });

  test("userConfig.root relative, should have configPath", () => {
    expect(() => {
      mergeFarmCliConfig({ root: "./path/to/" }, { root: "./path/userConfig" });
    }).toThrow();

    const result = mergeFarmCliConfig(
      { root: "./path/to/", configPath: process.cwd() },
      { root: "./path/userConfig" }
    );

    expect(result).toEqual({
      root: path.resolve(process.cwd(), "./path/userConfig")
    });
  });
});
