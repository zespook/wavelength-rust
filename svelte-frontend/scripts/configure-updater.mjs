import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const frontendRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const configPath = process.env.TAURI_CONFIG_PATH || path.join(frontendRoot, "src-tauri", "tauri.conf.json");
const config = JSON.parse(await readFile(configPath, "utf8"));
const repository = process.env.GITHUB_REPOSITORY || "";
const publicKey =
  process.env.TAURI_UPDATER_PUBLIC_KEY?.trim() ||
  config.plugins?.updater?.pubkey?.trim() ||
  "";
const signingPrivateKey = process.env.TAURI_SIGNING_PRIVATE_KEY || "";

if (!/^[\w.-]+\/[\w.-]+$/.test(repository)) {
  throw new Error("GITHUB_REPOSITORY must be set to owner/repository for signed releases.");
}
if (!publicKey.trim()) {
  throw new Error("TAURI_UPDATER_PUBLIC_KEY must contain the updater signing public key.");
}
if (!signingPrivateKey.trim()) {
  throw new Error("TAURI_SIGNING_PRIVATE_KEY must be configured for signed release builds.");
}

config.bundle = { ...config.bundle, createUpdaterArtifacts: true };
config.plugins = {
  ...config.plugins,
  updater: {
    pubkey: publicKey.trim(),
    endpoints: [
      `https://github.com/${repository}/releases/latest/download/latest.json`,
    ],
  },
};
await writeFile(configPath, `${JSON.stringify(config, null, 2)}\n`, "utf8");
console.log(`Configured signed updates for ${repository}.`);
