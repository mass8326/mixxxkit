interface TrackLocationPaths {
  directory: string;
  location: string;
}

export function applyTrackLocationReplacements(
  replacements: Map<string, string>,
  needle: TrackLocationPaths,
): TrackLocationPaths | undefined {
  for (const [from, to] of replacements.entries()) {
    if (needle.directory.startsWith(from)) {
      return {
        location: needle.location.replace(from, to),
        directory: needle.directory.replace(from, to),
      };
    }
  }
  return { location: needle.location, directory: needle.directory };
}

export function hasTrackLocationPaths(
  subject: {
    location: string | null;
    directory: string | null;
  } | null,
): subject is TrackLocationPaths {
  return !!(subject && subject.directory && subject.location);
}

export function applyReplacements(
  subject: string,
  replacements: Map<string, string>,
): string {
  for (const [from, to] of replacements.entries()) {
    if (subject.startsWith(from)) subject.replace(from, to);
  }
  return subject;
}
