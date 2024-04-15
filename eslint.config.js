import base from "@mgnsfr/eslint-config";
import { globifyGitIgnoreFile } from "globify-gitignore";

const globs = await globifyGitIgnoreFile("./");
const ignores = globs.map(({ glob, included }) => {
  if (!glob.startsWith("**/")) glob = "**/" + glob;
  return (included ? "!" : "") + glob;
});

/** @type {import("eslint").Linter.FlatConfig[]} */
const config = [
  { ignores },
  ...base,
  {
    rules: {
      "no-console": "off",
      "no-restricted-syntax": [
        "error",
        {
          selector:
            "CallExpression[callee.object.name='console'][callee.property.name='log']",
          message: "console.log() is for temporary development use only",
        },
      ],
    },
  },
];

export default config;
