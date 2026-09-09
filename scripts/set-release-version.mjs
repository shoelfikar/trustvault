import { readFile, writeFile } from 'node:fs/promises';

const [version] = process.argv.slice(2);
if (!version || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error('usage: node scripts/set-release-version.mjs <semver-without-v>');
  process.exit(2);
}

for (const path of ['package.json', 'src-tauri/tauri.conf.json']) {
  const document = JSON.parse(await readFile(path, 'utf8'));
  document.version = version;
  await writeFile(path, `${JSON.stringify(document, null, 2)}\n`);
}

const cargoPath = 'Cargo.toml';
const cargo = await readFile(cargoPath, 'utf8');
const versionPattern = /(\[workspace\.package\]\s*\nversion\s*=\s*)"[^"]+"/;
if (!versionPattern.test(cargo)) {
  throw new Error('workspace package version was not found in Cargo.toml');
}
const updated = cargo.replace(versionPattern, `$1"${version}"`);
await writeFile(cargoPath, updated);

console.log(`release version set to ${version}`);
