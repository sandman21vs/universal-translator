// scripts/sync-meta.mjs
import { readFileSync, writeFileSync } from 'node:fs';
const meta = JSON.parse(readFileSync(new URL('../app.meta.json', import.meta.url), 'utf8'));
const root = new URL('../', import.meta.url);
for (const file of ['package.json', 'package-lock.json', 'src-tauri/tauri.conf.json']) {
  const path = new URL(file, root);
  const data = JSON.parse(readFileSync(path, 'utf8'));
  if (file === 'src-tauri/tauri.conf.json') { data.productName = meta.name; data.identifier = meta.identifier; data.app.windows[0].title = meta.name; }
  else { data.version = meta.version; if (data.packages?.['']) data.packages[''].version = meta.version; }
  const serialized = JSON.stringify(data, null, 2) + '\n';
  if (readFileSync(path, 'utf8') !== serialized) writeFileSync(path, serialized);
}
const cargoPath = new URL('src-tauri/Cargo.toml', root);
const cargo = readFileSync(cargoPath, 'utf8');
const updated = cargo.replace(/(\[package\][\s\S]*?\nversion = )"[^"]+"/, `$1"${meta.version}"`);
if (cargo !== updated) writeFileSync(cargoPath, updated);
