#!/usr/bin/env node

const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

const isWin = process.platform === "win32";
const binName = isWin ? "cypher.exe" : "cypher";
const binPath = path.join(__dirname, "bin", binName);

if (!fs.existsSync(binPath)) {
  console.error(`Cypher CLI binary not found at ${binPath}. Please reinstall the package.`);
  process.exit(1);
}

const child = spawn(binPath, process.argv.slice(2), {
  stdio: "inherit",
});

child.on("close", (code) => {
  process.exit(code);
});

child.on("error", (err) => {
  console.error(`Failed to start Cypher CLI: ${err.message}`);
  process.exit(1);
});
