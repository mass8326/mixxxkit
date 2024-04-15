import { createInterface } from "node:readline/promises";
import chalk from "chalk";

type MaybePromise<T> = T | Promise<T>;

export type Validator = (val: string) => MaybePromise<string | boolean>;

export const readline = createInterface({
  input: process.stdin,
  output: process.stdout,
});

export async function question(opts: {
  message: string;
  default?: string;
  validate?: Validator;
}): Promise<string> {
  for (;;) {
    const answer = await readline.question(
      opts.default
        ? `${opts.message} ("${opts.default}"): `
        : `${opts.message}: `,
    );
    const result = answer.trim() || opts.default;
    if (!result) continue;
    const validation = await opts.validate?.(result);
    if (validation === undefined || validation === true) return result;
    console.info(chalk.red(validation || "Invalid input!"));
  }
}
