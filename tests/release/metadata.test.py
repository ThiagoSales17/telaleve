import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class ReleaseMetadata(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        for name in ('scripts/release-metadata.py', 'package.json', 'package-lock.json',
                     'src-tauri/tauri.conf.json', 'src-tauri/Cargo.toml', 'src-tauri/Cargo.lock'):
            target = self.root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / name, target)

    def run_check(self, tag=None):
        env = dict(os.environ)
        for key in ('GITHUB_OUTPUT', 'GITHUB_REF_TYPE', 'GITHUB_REF_NAME'):
            env.pop(key, None)
        if tag:
            env.update(GITHUB_REF_TYPE='tag', GITHUB_REF_NAME=tag)
        return subprocess.run(['python3', str(self.root / 'scripts/release-metadata.py')],
                              env=env, capture_output=True, text=True)

    def test_current_version_and_matching_tag(self):
        version = json.loads((self.root / 'package.json').read_text())['version']
        result = self.run_check('v' + version)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('tag=v' + version, result.stdout)

    def test_stable_and_beta_classification(self):
        original = json.loads((self.root / 'package.json').read_text())['version']
        files = [p for p in self.root.rglob('*') if p.is_file() and p.name != 'release-metadata.py']
        for version, expected in [('1.0.0', 'false'), ('1.1.0-beta.1', 'true')]:
            for path in files:
                path.write_text(path.read_text().replace(original, version))
            result = self.run_check('v' + version)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('prerelease=' + expected, result.stdout)
            original = version

    def test_mismatched_tag_is_rejected(self):
        self.assertNotEqual(self.run_check('v99.99.99').returncode, 0)

    def test_mismatched_lock_is_rejected(self):
        path = self.root / 'package-lock.json'
        data = json.loads(path.read_text())
        data['packages']['']['version'] = '99.0.0'
        path.write_text(json.dumps(data))
        self.assertNotEqual(self.run_check().returncode, 0)

    def test_non_semantic_version_is_rejected(self):
        path = self.root / 'package.json'
        data = json.loads(path.read_text())
        data['version'] = '0.2.0-beta.01'
        path.write_text(json.dumps(data))
        self.assertNotEqual(self.run_check().returncode, 0)


if __name__ == '__main__':
    unittest.main()
