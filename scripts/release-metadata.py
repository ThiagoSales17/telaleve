#!/usr/bin/env python3
"""Reject inconsistent release versions before compiling or publishing."""
import json
import os
from pathlib import Path
import re
import tomllib

root = Path(__file__).resolve().parent.parent
version = json.loads((root / 'package.json').read_text())['version']
number = r'(?:0|[1-9][0-9]*)'
identifier = rf'(?:{number}|[0-9]*[A-Za-z-][0-9A-Za-z-]*)'
pattern = rf'{number}\.{number}\.{number}(?:-{identifier}(?:\.{identifier})*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?'
if not re.fullmatch(pattern, version):
    raise SystemExit('Versão inválida: use versionamento semântico.')
versions = {
    'package-lock.json': json.loads((root / 'package-lock.json').read_text())['version'],
    'package-lock.json (root)': json.loads((root / 'package-lock.json').read_text())['packages']['']['version'],
    'tauri.conf.json': json.loads((root / 'src-tauri/tauri.conf.json').read_text())['version'],
    'Cargo.toml': tomllib.loads((root / 'src-tauri/Cargo.toml').read_text())['package']['version'],
    'Cargo.lock': next(p['version'] for p in tomllib.loads((root / 'src-tauri/Cargo.lock').read_text())['package'] if p['name'] == 'telaleve'),
}
for name, actual in versions.items():
    if actual != version:
        raise SystemExit(f'{name}: {actual} difere de package.json: {version}')
tag = f'v{version}'
if os.environ.get('GITHUB_REF_TYPE') == 'tag' and os.environ.get('GITHUB_REF_NAME') != tag:
    raise SystemExit(f'A tag deve ser {tag}, igual à versão dos arquivos do projeto.')
values = f'version={version}\ntag={tag}\nprerelease={str(version.startswith("0.") or "-" in version.split("+")[0]).lower()}\n'
if os.environ.get('GITHUB_OUTPUT'):
    with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
        output.write(values)
else:
    print(values, end='')
