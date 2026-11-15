import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptDir = fileURLToPath(new URL(".", import.meta.url));
const [version, ...assetSpecs] = process.argv.slice(2);
const repository = process.env.GITHUB_REPOSITORY;

if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version ?? "")) {
  throw new Error("Provide a semantic release version.");
}
if (!repository || !/^[\w.-]+\/[\w.-]+$/.test(repository)) {
  throw new Error("GITHUB_REPOSITORY must identify the release repository.");
}
if (assetSpecs.length === 0) throw new Error("At least one signed update asset is required.");

const appSource = readFileSync(resolve(scriptDir, "..", "dioxus-frontend", "src", "desktop.rs"), "utf8");
const embeddedKey = /const UPDATE_PUBLIC_KEY: &str = "([^"]+)";/.exec(appSource)?.[1];
if (!embeddedKey) throw new Error("Could not read the app's embedded update verification key.");

const signatureDir = mkdtempSync(join(tmpdir(), "wavelength-update-signatures-"));
try {
  const assets = assetSpecs.map(spec => {
    const separator = spec.indexOf("=");
    if (separator <= 0) throw new Error(`Invalid update asset specification: ${spec}`);
    const platform = spec.slice(0, separator);
    const path = resolve(spec.slice(separator + 1));
    if (!["windows-x86_64", "darwin-x86_64", "darwin-aarch64"].includes(platform)) {
      throw new Error(`Unsupported update platform: ${platform}`);
    }
    return {
      platform,
      path,
      signaturePath: join(signatureDir, `${basename(path)}.minisig`),
    };
  });

  const signer = spawnSync(
    process.platform === "win32" ? "cargo.exe" : "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      resolve(scriptDir, "update-signer", "Cargo.toml"),
      "--",
      ...assets.flatMap(({ path, signaturePath }) => [path, signaturePath]),
    ],
    {
      stdio: "inherit",
      env: { ...process.env, UPDATE_SIGNING_PUBLIC_KEY: embeddedKey.replace(/\\n/g, "\n") },
    },
  );
  if (signer.error) throw signer.error;
  if (signer.status !== 0) {
    throw new Error(`Update signing failed with exit code ${signer.status}.`);
  }

  const platforms = Object.fromEntries(assets.map(({ platform, path, signaturePath }) => [
    platform,
    {
      url: `https://github.com/${repository}/releases/download/v${version}/${basename(path)}`,
      signature: readFileSync(signaturePath, "utf8").trim(),
    },
  ]));
  writeFileSync(
    "latest.json",
    `${JSON.stringify({
      version,
      pub_date: new Date().toISOString(),
      platforms,
    }, null, 2)}\n`,
  );
} finally {
  rmSync(signatureDir, { recursive: true, force: true });
}
