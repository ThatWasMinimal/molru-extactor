import { execSync } from "child_process";
import fs from "fs";

function run(cmd) {
  console.log(`\n> ${cmd}`);
  execSync(cmd, { stdio: "inherit" });
}

function getCargoVersion() {
  const cargo = fs.readFileSync("src-tauri/Cargo.toml", "utf-8");
  const match = cargo.match(/^version\s*=\s*"(.*?)"/m);
  if (!match) {
    throw new Error("Could not find version in Cargo.toml");
  }
  return match[1];
}

function bumpVersion(typeOrVersion) {
  if (["patch", "minor", "major"].includes(typeOrVersion)) {
    run(`cd src-tauri && cargo set-version --bump ${typeOrVersion}`);
  } else {
    run(`cd src-tauri && cargo set-version ${typeOrVersion}`);
  }
}

const arg = process.argv[2];

if (!arg) {
  console.log("Usage:");
  console.log("  node scripts/release.mjs patch");
  console.log("  node scripts/release.mjs minor");
  console.log("  node scripts/release.mjs major");
  console.log("  node scripts/release.mjs 1.2.3");
  process.exit(1);
}

try {
  bumpVersion(arg);

  const version = getCargoVersion();
  const tag = `v${version}`;

  run(`git add .`);
  run(`git commit -m "release ${version}"`);
  run(`git tag ${tag}`);
  run(`git push --follow-tags`);

  console.log(`\n🚀 Released ${version}`);
} catch (err) {
  console.error("Release failed:", err.message);
  process.exit(1);
}