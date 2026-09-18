"""Exercise the shell's safety gates without accessing either real disk."""
import os
from pathlib import Path
import subprocess
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'config/zsh/bin/rclone-tower-safe'


class WorkflowTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='tower-workflow-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        for name in ('source', 'destination', 'home', 'bin'):
            (self.root / name).mkdir()
        self.events = self.root / 'events'
        diskutil = self.root / 'bin/diskutil'
        diskutil.write_text('''#!/usr/bin/env python3
import os,sys,plistlib,time
volume=sys.argv[-1]
if sys.argv[1]=='verifyVolume':
    with open(os.environ['EVENTS'],'a') as f: f.write('verify '+volume+'\\n')
    if os.environ.get('VERIFY_WAIT'): time.sleep(60)
    sys.exit(int(os.environ.get('VERIFY_EXIT','0')))
uuid='695E64CC-83EF-4155-AB6B-858E431468AC' if volume.endswith('source') else '215CF628-9286-4D5A-9802-21DB64342287'
sys.stdout.buffer.write(plistlib.dumps(dict(VolumeUUID=uuid,MountPoint=volume,WritableVolume=True)))
''')
        diskutil.chmod(0o700)
        rclone = self.root / 'bin/rclone'
        rclone.write_text('''#!/bin/sh
echo sync >> "$EVENTS"
exit "${SYNC_EXIT:-0}"
''')
        rclone.chmod(0o700)
        text = SCRIPT.read_text().replace('/Volumes/Tower Backup', str(self.root / 'destination'))
        text = text.replace('/Volumes/Tower', str(self.root / 'source'))
        text = text.replace('/usr/sbin/diskutil', str(diskutil))
        self.script = self.root / 'workflow'
        self.script.write_text(text)
        self.env = dict(os.environ, HOME=str(self.root / 'home'),
                        PATH=str(self.root / 'bin') + ':' + os.environ['PATH'],
                        EVENTS=str(self.events), RCLONE_TOWER_MIN_FREE_GIB='0',
                        RCLONE_TOWER_DASHBOARD='0')

    def run_script(self, *args, **env):
        result = subprocess.run(['zsh', '-f', str(self.script), *args],
                                env=dict(self.env, **env), capture_output=True, text=True)
        events = self.events.read_text().splitlines() if self.events.exists() else []
        return result, events

    def test_both_checks_precede_sync(self):
        result, events = self.run_script()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([x.split()[0] for x in events], ['verify', 'verify', 'sync'])

    def test_failed_check_prevents_sync(self):
        result, events = self.run_script(VERIFY_EXIT='7')
        self.assertEqual(result.returncode, 7)
        self.assertEqual(len(events), 1)
        self.assertIn('Sync was not started', result.stderr)

    def test_check_only_never_syncs(self):
        result, events = self.run_script('--check-only')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(events), 2)
        self.assertTrue(all(x.startswith('verify ') for x in events))

    def test_dry_run_skips_filesystem_check(self):
        result, events = self.run_script('--dry-run')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(events, ['sync'])

    def test_sync_failure_propagates(self):
        result, events = self.run_script(SYNC_EXIT='9')
        self.assertEqual(result.returncode, 9)
        self.assertIn('history was not pruned', result.stderr)

    def test_cancel_verification_releases_lock_and_never_syncs(self):
        p = subprocess.Popen(['zsh', '-f', str(self.script)],
                             env=dict(self.env, VERIFY_WAIT='1'),
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            deadline = time.monotonic() + 5
            while not self.events.exists() and time.monotonic() < deadline:
                time.sleep(0.05)
            self.assertTrue(self.events.exists(), 'verification did not start')
            p.terminate()
            p.communicate(timeout=5)
            self.assertEqual(p.returncode, 143)
            self.assertNotIn('sync', self.events.read_text())
            result, _ = self.run_script('--check-only')
            self.assertEqual(result.returncode, 0, result.stderr)
        finally:
            if p.poll() is None:
                p.kill()
                p.communicate()


if __name__ == '__main__':
    unittest.main()
