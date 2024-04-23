import { relations } from "drizzle-orm";
import { cues, library, trackLocations } from "./tables";

export const cueRelations = relations(cues, ({ one }) => ({
  track: one(library, {
    fields: [cues.trackId],
    references: [library.id],
  }),
}));

export const libraryRelations = relations(library, ({ one, many }) => ({
  cues: many(cues),
  trackLocation: one(trackLocations, {
    fields: [library.location],
    references: [trackLocations.id],
  }),
}));
