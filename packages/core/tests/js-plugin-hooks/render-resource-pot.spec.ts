import { pathToFileURL } from "url";
import { expect, test } from "vitest";
import { getCompiler, getOutputFilePath } from "./common.js";

test("Js Plugin Execution - renderResourcePot", async () => {
  const hookName = "render-resource-pot";
  const calledHooks: string[] = [];
  const compiler = await getCompiler(
    "",
    [
      {
        name: "test-render-resource-pot",
        priority: 1000,
        renderResourcePot: {
          filters: {
            moduleIds: ["^index.ts\\?foo=bar$"],
            resourcePotTypes: ["js"]
          },
          executor: async (param) => {
            expect(param.content).toContain("render-resource-pot-return-value");
            expect(param.sourceMapChain).toEqual([]);
            console.log(param.resourcePotInfo);
            if (
              param.resourcePotInfo.modules["index.ts?foo=bar"]
                .originalLength == 52
            ) {
              param.resourcePotInfo.modules["index.ts?foo=bar"].originalLength =
                51;
            }
            expect(param.resourcePotInfo).matchSnapshot();
            calledHooks.push("renderResourcePot");
            return {
              content: param.content.replace(
                "render-resource-pot-return-value",
                "1"
              )
            };
          }
        }
      }
    ],
    hookName
  );

  await compiler.compile();
  await compiler.writeResourcesToDisk();

  expect(calledHooks).toEqual(["renderResourcePot"]);

  const outputFilePath = getOutputFilePath("", hookName);

  if (process.platform === "win32") {
    const result = await import(pathToFileURL(outputFilePath).toString());
    expect(result.default).toBe("1");
  } else {
    const result = await import(outputFilePath);
    expect(result.default).toBe("1");
  }
});
