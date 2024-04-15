import type { Config } from "drizzle-kit";

export default {
  schema: "./src/schema.ts",
  out: "./introspection",
  driver: "better-sqlite",
  dbCredentials: {
    url: "./source.sqlite",
  },
} satisfies Config;
