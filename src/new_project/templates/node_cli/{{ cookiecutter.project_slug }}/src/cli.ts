import { Command } from "commander";

import { greet } from "./app.js";
import { configPath, loadConfig } from "./config.js";

export function buildProgram(): Command {
  const program = new Command();

  program
    .name("{{ cookiecutter.command_name }}")
    .description("{{ cookiecutter.description }}")
    .showHelpAfterError();

  program
    .command("greet")
    .description("Print one or more greetings")
    .option("--name <name>", "Name to greet")
    .option("--count <count>", "Number of greetings to print", parseCount)
    .action(async (options: { name?: string; count?: number }) => {
      const config = await loadConfig();

      greet({
        name: options.name ?? config.name,
        count: options.count ?? config.count,
      });
    });

  program
    .command("config-path")
    .description("Print the resolved config path")
    .action(() => {
      console.log(configPath());
    });

  return program;
}

function parseCount(value: string): number {
  const parsed = Number.parseInt(value, 10);

  if (Number.isNaN(parsed) || parsed < 1) {
    throw new Error("count must be a positive integer");
  }

  return parsed;
}
