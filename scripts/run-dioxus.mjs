import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = dirname(fileURLToPath(import.meta.url));
const action = process.argv[2] ?? "dev";
if (!["dev", "build"].includes(action)) {
  throw new Error("Usage: node scripts/run-dioxus.mjs [dev|build]");
}

let command;
let args;
if (process.platform === "win32") {
  command = "powershell.exe";
  args = ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", join(root, "run-dioxus.ps1"), action];
} else if (process.platform === "darwin") {
  command = "bash";
  args = [join(root, "run-dioxus.sh"), action];
} else {
  throw new Error("Native Dioxus Desktop scripts are configured for Windows and macOS.");
}

const result = spawnSync(command, args, { stdio: "inherit" });
if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
