import type { Validator } from "./question";
import { Database, SQLiteError } from "bun:sqlite";

export const validateDatbasePath: Validator = async function (path: string) {
  if (!path) return "A database is required!";
  try {
    new Database(path, { readonly: true }).close();
    return true;
  } catch (err: unknown) {
    if (err instanceof SQLiteError && err.code === "SQLITE_CANTOPEN")
      return "Unable to open database at that path!";
    else throw err;
  }
};
