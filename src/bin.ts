import { Database } from "bun:sqlite";
import { drizzle } from "drizzle-orm/bun-sqlite";
import {
  promptForDatabases,
  promptForInitialization,
  promptForReplacements,
} from "$lib/cli";
import { mergeLibraries } from "$lib/core";
import { schema } from "$lib/schema";

{
  await promptForInitialization();
  const paths = await promptForDatabases();
  using sourceSql = new Database(paths.source, { readonly: true });
  const sourceDb = drizzle(sourceSql, { schema });
  const replacements = await promptForReplacements(sourceDb);

  const file = Bun.file(paths.target);
  await Bun.write(paths.output, file);
  using targetSql = new Database(paths.output, { readwrite: true });
  const targetDb = drizzle(targetSql, { schema });

  console.info("Merging...");
  await mergeLibraries(sourceDb, targetDb, replacements);
}

console.info("Done!");
process.exit(0);
