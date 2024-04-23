import { stat } from "node:fs/promises";
import { normalize } from "node:path";
import { Database } from "bun:sqlite";
import { drizzle } from "drizzle-orm/bun-sqlite";
import * as schema from "./schema";
import { question, readline } from "./util/question";
import {
  hasTrackLocationPaths,
  replaceTrackLocationPaths,
} from "./util/replace";
import { validateDatbasePath } from "./util/validate";

// TODO: Remove when issue resolved - https://github.com/oven-sh/bun/issues/5953
console.info("Press enter to initialize...");
await readline.question("");

const sourcePath = await question({
  message: "Path to source database",
  default: "./source.sqlite",
  validate: validateDatbasePath,
});
const sourceSql = new Database(sourcePath, { readonly: true });
const sourceDb = drizzle(sourceSql, { schema });
const dirs = await sourceDb
  .select({ dir: schema.directories.directory })
  .from(schema.directories);
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
const tracks = await sourceDb.query.library.findMany({
  with: { cues: true, trackLocation: true },
});
sourceSql.close();

const targetPath = await question({
  message: "Path to target database",
  default: "./target.sqlite",
  validate: validateDatbasePath,
});
const outputPath = await question({
  message: "Path to output database",
  default: "./mixxxdb.sqlite",
});

console.info("Processing...");
const file = Bun.file(targetPath);
await Bun.write(outputPath, file);

const targetSql = new Database(outputPath, { readwrite: true });
const targetDb = drizzle(targetSql, { schema });
await targetDb.transaction(async (tx) => {
  await tx
    .insert(schema.directories)
    .values([...replacements.values()].map((directory) => ({ directory })));

  for (const [i, track] of Object.entries(tracks)) {
    if (!hasTrackLocationPaths(track.trackLocation)) {
      console.warn(`Skipping #${i} "${track.artist} - ${track.title}"`);
      continue;
    }
    const [{ locationId }] = await tx
      .insert(schema.trackLocations)
      .values({
        ...track.trackLocation,
        ...replaceTrackLocationPaths(replacements, track.trackLocation),
        id: undefined,
      })
      .returning({ locationId: schema.trackLocations.id });
    const [{ trackId }] = await tx
      .insert(schema.library)
      .values({ ...track, id: undefined, location: locationId })
      .returning({ trackId: schema.library.id });
    if (track.cues.length) {
      await tx
        .insert(schema.cues)
        .values(track.cues.map((cue) => ({ ...cue, id: undefined, trackId })));
    }
  }
});
targetSql.close();

console.info("Done!");
process.exit(0);
