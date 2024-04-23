export * from "./relations";
export * from "./tables";

import type { BunSQLiteDatabase } from "drizzle-orm/bun-sqlite";
import * as relations from "./relations";
import * as tables from "./tables";

export const schema = { ...relations, ...tables };

export type DrizzleDb = BunSQLiteDatabase<typeof schema>;
