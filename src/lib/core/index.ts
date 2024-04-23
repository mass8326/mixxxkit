import type { DrizzleDb } from "$lib/schema";
import * as remeda from "remeda";
import {
  applyReplacements,
  applyTrackLocationReplacements,
  hasTrackLocationPaths,
} from "./replace";
import { cues, directories, library, trackLocations } from "$lib/schema";

export async function mergeLibraries(
  source: DrizzleDb,
  target: DrizzleDb,
  replacements: Map<string, string>,
) {
  const dirs = await source
    .select({ dir: directories.directory })
    .from(directories);
  const tracks = await source.query.library.findMany({
    with: { cues: true, trackLocation: true },
  });

  await target.transaction(async (tx) => {
    await tx.insert(directories).values(
      dirs
        .map(({ dir }) => dir)
        .filter(remeda.isNonNullish)
        .map((dir) => ({ directory: applyReplacements(dir, replacements) })),
    );

    for (const [i, track] of Object.entries(tracks)) {
      if (!hasTrackLocationPaths(track.trackLocation)) {
        console.warn(`Skipping #${i} "${track.artist} - ${track.title}"`);
        continue;
      }
      const [{ locationId }] = await tx
        .insert(trackLocations)
        .values({
          ...track.trackLocation,
          ...applyTrackLocationReplacements(replacements, track.trackLocation),
          id: undefined,
        })
        .returning({ locationId: trackLocations.id });
      const [{ trackId }] = await tx
        .insert(library)
        .values({ ...track, id: undefined, location: locationId })
        .returning({ trackId: library.id });
      if (track.cues.length) {
        await tx
          .insert(cues)
          .values(
            track.cues.map((cue) => ({ ...cue, id: undefined, trackId })),
          );
      }
    }
  });
}
