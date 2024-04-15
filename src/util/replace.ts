interface TrackLocationPaths {
  directory: string;
  location: string;
}

export function replaceTrackLocationPaths(
  replacements: Map<string, string>,
  needle: TrackLocationPaths,
): TrackLocationPaths | undefined {
  const entries = [...replacements.entries()];
  for (const [from, to] of entries) {
    if (needle.directory.startsWith(from)) {
      return {
        location: needle.location?.replace(from, to),
        directory: needle.directory?.replace(from, to),
      };
    }
  }
}
export function hasTrackLocationPaths(
  subject: {
    location: string | null;
    directory: string | null;
  } | null,
): subject is TrackLocationPaths {
  return !!(subject && subject.directory && subject.location);
}
