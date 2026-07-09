const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const pkg = require("./package.json");
const REPO = "Ayan-Flash/Cypher";
const VERSION = "v" + pkg.version;

function detectTarget() {
  const os = process.platform;
  const arch = process.arch;
  if (os === "win32") return { os: "windows", ext: ".zip" };
  if (os === "darwin") return { os: "macos", ext: ".tar.gz" };
  if (os === "linux") return { os: "linux", ext: ".tar.gz" };
  throw new Error(`Unsupported OS: ${os}`);
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const http = url.startsWith("https") ? require("https") : require("http");
    const file = fs.createWriteStream(dest);
    http.get(url, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        file.close(); fs.unlinkSync(dest);
        return download(res.headers.location, dest).then(resolve).catch(reject);
      }
      if (res.statusCode !== 200) {
        file.close(); fs.unlinkSync(dest);
        reject(new Error(`HTTP ${res.statusCode}`)); return;
      }
      res.pipe(file);
      file.on("finish", () => { file.close(); resolve(); });
    }).on("error", (err) => {
      file.close(); fs.unlinkSync(dest, () => {}); reject(err);
    });
  });
}

async function extractZip(zipPath, extractDir, binName) {
  const AdmZip = require("adm-zip");
  const zip = new AdmZip(zipPath);
  const entry = zip.getEntries().find((e) => e.entryName.endsWith(".exe"));
  if (!entry) throw new Error("No .exe found in zip");
  fs.writeFileSync(path.join(extractDir, binName), entry.getData());
}

async function extractTarGz(tarPath, extractDir, binName) {
  const tar = require("tar");
  const files = [];
  await tar.extract({
    file: tarPath, cwd: extractDir,
    filter: (p) => { if (p.endsWith("cypher")) files.push(p); return p.endsWith("cypher"); },
  });
  if (files.length === 0) throw new Error("No 'cypher' binary found in archive");
  const fullPath = path.join(extractDir, files[0]);
  if (fullPath !== path.join(extractDir, binName)) {
    fs.renameSync(fullPath, path.join(extractDir, binName));
  }
}

async function install() {
  const binDir = path.join(__dirname, "bin");
  fs.mkdirSync(binDir, { recursive: true });

  const isWin = process.platform === "win32";
  const binName = isWin ? "cypher.exe" : "cypher";
  const binPath = path.join(binDir, binName);

  if (fs.existsSync(binPath)) {
    console.log(`[cypher-cli] Binary already exists at ${binPath}`);
    console.log(`[cypher-cli] Run \`cypher --version\` to verify`);
    return;
  }

  const { os, ext } = detectTarget();
  const arch = process.arch === "x64" ? "x86_64" : "aarch64";
  const target = `${os}-${arch}`;

  // Try download from GitHub release
  const url = `https://github.com/${REPO}/releases/download/${VERSION}/cypher-${target}${ext}`;
  const tmp = path.join(binDir, `pkg${ext}`);

  console.log(`[cypher-cli] Downloading ${url}...`);
  try {
    await download(url, tmp);
    if (ext === ".zip") await extractZip(tmp, binDir, binName);
    else await extractTarGz(tmp, binDir, binName);
    fs.unlinkSync(tmp);
  } catch (e) {
    if (fs.existsSync(tmp)) fs.unlinkSync(tmp);
    // Release not available — build from source instead
    console.log(`[cypher-cli] Pre-built binary unavailable. Building from source...`);
    console.log(`[cypher-cli] This requires Rust. Installing via cargo...`);
    const rootFlag = isWin ? "--root" : "--root";
    execSync(`cargo install --git https://github.com/${REPO}.git ${rootFlag} "${binDir}"`, {
      stdio: "inherit",
    });
    const cargoBinDir = path.join(binDir, "bin");
    const cargoBin = path.join(cargoBinDir, binName);
    if (fs.existsSync(cargoBin)) {
      fs.renameSync(cargoBin, binPath);
      fs.rmSync(cargoBinDir, { recursive: true, force: true });
    }
  }

  if (!isWin) fs.chmodSync(binPath, 0o755);
  console.log(`[cypher-cli] Installed successfully: ${binPath}`);
}

install().catch((err) => {
  console.error(`[cypher-cli] Installation failed: ${err.message}`);
  process.exit(1);
});
