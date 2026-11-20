import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const dioxusFrontend = join(root, "dioxus-frontend");
const dioxusConfig = readFileSync(join(dioxusFrontend, "Dioxus.toml"), "utf8");
const applicationName = dioxusConfig.match(/^name\s*=\s*"([^"]+)"/m)?.[1];

if (!applicationName) {
  throw new Error("Could not read the Dioxus application name from Dioxus.toml.");
}

const dx = process.platform === "win32" ? "dx.exe" : "dx";
const build = spawnSync(
  dx,
  ["build", "--platform", "web", "--release", "--no-default-features", "--features", "web"],
  {
  cwd: dioxusFrontend,
  stdio: "inherit",
  },
);

if (build.error) throw build.error;
if (build.status !== 0) process.exit(build.status ?? 1);

const buildOutput = join(
  dioxusFrontend,
  "target",
  "dx",
  applicationName,
  "release",
  "web",
  "public",
);
if (!existsSync(buildOutput)) {
  throw new Error(`Dioxus build output was not found at ${buildOutput}.`);
}

const desktopOutput = join(root, "dist-dioxus");
mkdirSync(desktopOutput, { recursive: true });
for (const name of readdirSync(buildOutput)) {
  cpSync(join(buildOutput, name), join(desktopOutput, name), {
    force: true,
    recursive: true,
  });
}
