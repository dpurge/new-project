import { readFile } from "node:fs/promises";
import { homedir } from "node:os";
import { join } from "node:path";

export type Config = {
  name?: string;
  count?: number;
};

export function configPath(): string {
  return join(homedir(), ".config", "{{ cookiecutter.project_slug }}", "config.env");
}

export async function loadConfig(): Promise<Config> {
  try {
    const content = await readFile(configPath(), "utf-8");
    return parseConfig(content);
  } catch (error) {
    if (isMissingFile(error)) {
      return {};
    }

    throw error;
  }
}

function parseConfig(content: string): Config {
  const config: Config = {};

  for (const line of content.split(/\r?\n/u)) {
    const trimmed = line.trim();
    if (trimmed === "" || trimmed.startsWith("#")) {
      continue;
    }

    const separatorIndex = trimmed.indexOf("=");
    if (separatorIndex === -1) {
      continue;
    }

    const key = trimmed.slice(0, separatorIndex).trim();
    const value = trimmed.slice(separatorIndex + 1).trim();

    if (key === "name") {
      config.name = value;
      continue;
    }

    if (key === "count") {
      const parsed = Number.parseInt(value, 10);
      if (Number.isNaN(parsed)) {
        throw new Error("Invalid count in config file");
      }
      config.count = parsed;
    }
  }

  return config;
}

function isMissingFile(error: unknown): error is NodeJS.ErrnoException {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    error.code === "ENOENT"
  );
}
