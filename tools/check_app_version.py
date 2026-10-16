"""Keep the three packaging manifests aligned without creating a UI constant."""
import json
import pathlib
import re

root = pathlib.Path(__file__).resolve().parents[1]
frontend = json.loads((root / 'package.json').read_text())['version']
tauri = json.loads((root / 'src-tauri/tauri.conf.json').read_text())['version']
cargo = re.search(r'^version\s*=\s*"([^"]+)"', (root / 'src-tauri/Cargo.toml').read_text(), re.M).group(1)
if len({frontend, tauri, cargo}) != 1:
    raise SystemExit(f'Application version mismatch: package.json={frontend}, tauri.conf.json={tauri}, Cargo.toml={cargo}')
print(f'Application version consistent: {frontend}')
