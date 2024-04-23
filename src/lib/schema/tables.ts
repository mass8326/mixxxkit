import { sql } from "drizzle-orm";
import {
  blob,
  index,
  integer,
  numeric,
  real,
  sqliteTable,
  text,
} from "drizzle-orm/sqlite-core";

export const settings = sqliteTable("settings", {
  name: text("name").notNull(),
  value: text("value"),
  locked: integer("locked").default(0),
  hidden: integer("hidden").default(0),
});

export const trackLocations = sqliteTable("track_locations", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  location: numeric("location"),
  filename: numeric("filename"),
  directory: numeric("directory"),
  filesize: integer("filesize"),
  fsDeleted: integer("fs_deleted"),
  needsVerification: integer("needs_verification"),
});

export const libraryHashes = sqliteTable("LibraryHashes", {
  directoryPath: numeric("directory_path").primaryKey(),
  hash: integer("hash"),
  directoryDeleted: integer("directory_deleted"),
  needsVerification: integer("needs_verification").default(0),
});

export const playlists = sqliteTable("Playlists", {
  id: integer("id").primaryKey(),
  name: numeric("name"),
  position: integer("position"),
  hidden: integer("hidden").default(0).notNull(),
  dateCreated: numeric("date_created"),
  dateModified: numeric("date_modified"),
  locked: integer("locked").default(0),
});

export const playlistTracks = sqliteTable(
  "PlaylistTracks",
  {
    id: integer("id").primaryKey(),
    playlistId: integer("playlist_id").references(() => playlists.id),
    trackId: integer("track_id").references(() => library.id),
    position: integer("position"),
    plDatetimeAdded: text("pl_datetime_added"),
  },
  (table) => {
    return {
      idxPlaylistTracksTrackId: index("idx_PlaylistTracks_track_id").on(
        table.trackId,
      ),
      idxPlaylistTracksPlaylistIdTrackId: index(
        "idx_PlaylistTracks_playlist_id_track_id",
      ).on(table.playlistId, table.trackId),
    };
  },
);

export const cues = sqliteTable("cues", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  trackId: integer("track_id")
    .notNull()
    .references(() => library.id),
  type: integer("type").default(0).notNull(),
  position: integer("position").default(-1).notNull(),
  length: integer("length").default(0).notNull(),
  hotcue: integer("hotcue").default(-1).notNull(),
  label: text("label").default("").notNull(),
  color: integer("color").default(4294901760).notNull(),
});

export const crates = sqliteTable("crates", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  name: numeric("name").notNull(),
  count: integer("count").default(0),
  show: integer("show").default(1),
  locked: integer("locked").default(0),
  autodjSource: integer("autodj_source").default(0),
});

export const crateTracks = sqliteTable(
  "crate_tracks",
  {
    crateId: integer("crate_id")
      .notNull()
      .references(() => crates.id),
    trackId: integer("track_id")
      .notNull()
      .references(() => library.id),
  },
  (table) => {
    return {
      idxCrateTracksTrackId: index("idx_crate_tracks_track_id").on(
        table.trackId,
      ),
    };
  },
);

export const library = sqliteTable("library", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  artist: numeric("artist"),
  title: numeric("title"),
  album: numeric("album"),
  year: numeric("year"),
  genre: numeric("genre"),
  tracknumber: numeric("tracknumber"),
  location: integer("location").references(() => trackLocations.location),
  comment: numeric("comment"),
  url: numeric("url"),
  duration: real("duration"),
  bitrate: integer("bitrate"),
  samplerate: integer("samplerate"),
  cuepoint: integer("cuepoint"),
  bpm: real("bpm"),
  wavesummaryhex: blob("wavesummaryhex"),
  channels: integer("channels"),
  datetimeAdded: text("datetime_added").default("sql`(CURRENT_TIMESTAMP)`"),
  mixxxDeleted: integer("mixxx_deleted"),
  played: integer("played"),
  headerParsed: integer("header_parsed").default(0),
  filetype: numeric("filetype").default(sql`("?")`),
  replaygain: real("replaygain"),
  timesplayed: integer("timesplayed").default(0),
  rating: integer("rating").default(0),
  key: numeric("key").default(sql`("")`),
  beats: blob("beats"),
  beatsVersion: text("beats_version"),
  composer: numeric("composer").default(sql`("")`),
  bpmLock: integer("bpm_lock").default(0),
  beatsSubVersion: text("beats_sub_version").default(""),
  keys: blob("keys"),
  keysVersion: text("keys_version"),
  keysSubVersion: text("keys_sub_version"),
  keyId: integer("key_id").default(0),
  grouping: text("grouping").default(""),
  albumArtist: text("album_artist").default(""),
  coverartSource: integer("coverart_source").default(0),
  coverartType: integer("coverart_type").default(0),
  coverartLocation: text("coverart_location").default(""),
  coverartHash: integer("coverart_hash").default(0),
  replaygainPeak: real("replaygain_peak").default(-1),
  tracktotal: text("tracktotal").default("//"),
  color: integer("color"),
  coverartColor: integer("coverart_color"),
  coverartDigest: blob("coverart_digest"),
  lastPlayedAt: numeric("last_played_at").default(sql`(NULL)`),
  sourceSynchronizedMs: integer("source_synchronized_ms").default(sql`(NULL)`),
});

export const itunesLibrary = sqliteTable("itunes_library", {
  id: integer("id").primaryKey(),
  artist: numeric("artist"),
  title: numeric("title"),
  album: numeric("album"),
  year: numeric("year"),
  genre: numeric("genre"),
  tracknumber: numeric("tracknumber"),
  location: numeric("location"),
  comment: numeric("comment"),
  duration: integer("duration"),
  bitrate: integer("bitrate"),
  bpm: integer("bpm"),
  rating: integer("rating"),
  grouping: text("grouping").default(""),
  albumArtist: text("album_artist").default(""),
});

export const itunesPlaylists = sqliteTable("itunes_playlists", {
  id: integer("id").primaryKey(),
  name: numeric("name"),
});

export const itunesPlaylistTracks = sqliteTable("itunes_playlist_tracks", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  playlistId: integer("playlist_id").references(() => itunesPlaylists.id),
  trackId: integer("track_id").references(() => itunesLibrary.id),
  position: integer("position").default(0),
});

export const traktorLibrary = sqliteTable("traktor_library", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  artist: numeric("artist"),
  title: numeric("title"),
  album: numeric("album"),
  year: numeric("year"),
  genre: numeric("genre"),
  tracknumber: numeric("tracknumber"),
  location: numeric("location"),
  comment: numeric("comment"),
  duration: integer("duration"),
  bitrate: integer("bitrate"),
  bpm: real("bpm"),
  key: numeric("key"),
  rating: integer("rating"),
});

export const traktorPlaylists = sqliteTable("traktor_playlists", {
  id: integer("id").primaryKey(),
  name: numeric("name"),
});

export const traktorPlaylistTracks = sqliteTable("traktor_playlist_tracks", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  playlistId: integer("playlist_id").references(() => traktorPlaylists.id),
  trackId: integer("track_id").references(() => traktorLibrary.id),
  position: integer("position").default(0),
});

export const rhythmboxLibrary = sqliteTable("rhythmbox_library", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  artist: numeric("artist"),
  title: numeric("title"),
  album: numeric("album"),
  year: numeric("year"),
  genre: numeric("genre"),
  tracknumber: numeric("tracknumber"),
  location: numeric("location"),
  comment: numeric("comment"),
  duration: integer("duration"),
  bitrate: integer("bitrate"),
  bpm: real("bpm"),
  key: numeric("key"),
  rating: integer("rating"),
});

export const rhythmboxPlaylists = sqliteTable("rhythmbox_playlists", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  name: numeric("name"),
});

export const rhythmboxPlaylistTracks = sqliteTable(
  "rhythmbox_playlist_tracks",
  {
    id: integer("id").primaryKey({ autoIncrement: true }),
    playlistId: integer("playlist_id").references(() => rhythmboxPlaylists.id),
    trackId: integer("track_id").references(() => rhythmboxLibrary.id),
    position: integer("position").default(0),
  },
);

export const trackAnalysis = sqliteTable(
  "track_analysis",
  {
    id: integer("id").primaryKey({ autoIncrement: true }),
    trackId: integer("track_id")
      .notNull()
      .references(() => trackLocations.id),
    type: numeric("type"),
    description: numeric("description"),
    version: numeric("version"),
    created: text("created").default("sql`(CURRENT_TIMESTAMP)`"),
    dataChecksum: numeric("data_checksum"),
  },
  (table) => {
    return {
      trackIdIdx: index("track_analysis_track_id_index").on(table.trackId),
    };
  },
);

export const directories = sqliteTable("directories", {
  directory: text("directory"),
});
