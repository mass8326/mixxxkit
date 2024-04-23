import { stat } from "node:fs/promises";
import { normalize } from "node:path";
import { question, readline } from "./question";
import { validateDatbasePath } from "./validate";
import { type DrizzleDb, directories } from "$lib/schema";

// TODO: Remove when issue resolved - https://github.com/oven-sh/bun/issues/5953
export async function promptForInitialization() {
  console.info("Press enter to initialize...");
  await readline.question("");
}

export async function promptForDatabases() {
  const source = await question({
    message: "Path to source database",
    default: "./source.sqlite",
    validate: validateDatbasePath,
  });
  const target = await question({
    message: "Path to target database",
    default: "./target.sqlite",
    validate: validateDatbasePath,
  });
  const output = await question({
    message: "Path to output database",
    default: "./mixxxdb.sqlite",
  });
  return { source, target, output };
}

export async function promptForReplacements(db: DrizzleDb) {
  const dirs = await db
    .select({ dir: directories.directory })
    .from(directories);
  const replacements = new Map<string, string>();
  for (const { dir } of dirs) {
    if (!dir) continue;
    const raw = await question({
      message: `New path to "${dir}"`,
      validate: async (val) => {
        if (!val) return "Path is required!";
        const check = await stat(val).catch(() => null);
        return check?.isDirectory() ? true : "Path is not a valid directory!";
      },
    });
    const path = normalize(raw).replaceAll("\\", "/");
    replacements.set(dir, path);
  }
  return replacements;
}
