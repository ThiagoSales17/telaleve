"""Exercise dependency selection and diagnostic failure modes without installation."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]

class LinuxScripts(unittest.TestCase):
    def test_distribution_families_and_derivatives(self):
        examples = [('debian', '', 'apt-get'), ('ubuntu', 'debian', 'apt-get'),
                    ('linuxmint', 'ubuntu debian', 'apt-get'), ('fedora', '', 'dnf'),
                    ('arch', '', 'pacman'), ('manjaro', 'arch', 'pacman'),
                    ('opensuse-tumbleweed', 'opensuse suse', 'zypper')]
        with tempfile.TemporaryDirectory() as folder:
            release = Path(folder) / 'os-release'
            for name, parent, manager in examples:
                with self.subTest(name=name):
                    release.write_text(f'ID={name}\nID_LIKE="{parent}"\n')
                    env = {**os.environ, 'TELALEVE_OS_RELEASE': str(release)}
                    result = subprocess.run(['sh', str(ROOT/'scripts/install-deps.sh'), '--dry-run'], env=env, capture_output=True, text=True)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.assertTrue(result.stdout.startswith(manager + ' '), result.stdout)
                    self.assertIn('xdg-desktop-portal', result.stdout)

    def test_unknown_distribution_points_to_flatpak(self):
        with tempfile.TemporaryDirectory() as folder:
            release=Path(folder)/'os-release';release.write_text('ID=unknown\n')
            result=subprocess.run(['sh',str(ROOT/'scripts/install-deps.sh'),'--dry-run'],env={**os.environ,'TELALEVE_OS_RELEASE':str(release)},capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0)
            self.assertIn('Flatpak',result.stderr)

    def diagnose(self, session, missing='', portal_ok=True, display=':test', flatpak=''):
        with tempfile.TemporaryDirectory() as folder:
            gst=Path(folder)/'gst-inspect-1.0'
            gst.write_text('#!/bin/sh\n[ "$1" != "$TELALEVE_MISSING" ]\n');gst.chmod(0o755)
            dbus=Path(folder)/'gdbus'
            dbus.write_text('#!/bin/sh\n' + ('echo org.freedesktop.portal.ScreenCast\n' if portal_ok else 'exit 1\n'));dbus.chmod(0o755)
            env={**os.environ,'PATH':folder+os.pathsep+os.environ['PATH'],'XDG_SESSION_TYPE':session,'WAYLAND_DISPLAY':'','FLATPAK_ID':flatpak,'DISPLAY':display,'TELALEVE_MISSING':missing}
            return subprocess.run(['sh',str(ROOT/'scripts/diagnose.sh')],env=env,capture_output=True,text=True)

    def test_x11_plugin_missing_is_failure(self):
        result=self.diagnose('x11','ximagesrc')
        self.assertNotEqual(result.returncode,0)
        self.assertIn('AUSENTE ximagesrc',result.stdout)

    def test_wayland_does_not_require_x11(self):
        result=self.diagnose('wayland',display='')
        self.assertEqual(result.returncode,0,result.stdout)
        self.assertIn('OK pipewiresrc',result.stdout)
        self.assertNotIn('ximagesrc',result.stdout)

    def test_x11_flatpak_does_not_require_a_wayland_portal(self):
        result=self.diagnose('x11',portal_ok=False,flatpak='io.github.ThiagoSales17.TelaLeve')
        self.assertEqual(result.returncode,0,result.stdout)
        self.assertIn('OK ximagesrc',result.stdout)
        self.assertNotIn('AUSENTE portal',result.stdout)

    def test_wayland_without_portal_is_failure(self):
        result=self.diagnose('wayland',portal_ok=False,display='')
        self.assertNotEqual(result.returncode,0)
        self.assertIn('AUSENTE portal ScreenCast',result.stdout)

if __name__ == '__main__':
    unittest.main()
