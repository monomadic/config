"""Disposable integration fixtures for cleanup's data-loss guards."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

HELPER = Path(__file__).resolve().parents[2] / 'bin/lib/cleanup-safety.py'
spec = importlib.util.spec_from_file_location('cleanup_safety', HELPER)
safety = importlib.util.module_from_spec(spec)
spec.loader.exec_module(safety)


class SafetyTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='cleanup-test-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()
        self.remote = self.root / 'origin.git'
        self.repo = self.root / 'checkout'
        self.git(self.root, 'init', '--bare', str(self.remote))
        self.git(self.root, 'clone', str(self.remote), str(self.repo))
        self.git(self.repo, 'config', 'user.name', 'Cleanup Test')
        self.git(self.repo, 'config', 'user.email', 'cleanup@example.invalid')
        (self.repo / 'source').write_text('original')
        (self.repo / '.gitignore').write_text('runtime/\n')
        self.git(self.repo, 'add', '.')
        self.git(self.repo, 'commit', '-m', 'base')
        self.git(self.repo, 'push', 'origin', 'HEAD')
        self.branch = self.git(self.repo, 'branch', '--show-current').strip()

    def git(self, repo, *args):
        return subprocess.check_output(['git', '-C', str(repo), *args], stderr=subprocess.DEVNULL, text=True)

    def refused(self):
        with self.assertRaises(ValueError):
            safety.check(self.repo)
        self.assertTrue((self.repo / 'source').exists())

    def commit_unique(self):
        (self.repo / 'source').write_text('unique')
        self.git(self.repo, 'commit', '-am', 'unique')

    def test_clean_origin_recoverable(self):
        self.assertEqual(safety.check(self.repo), str(self.remote))

    def test_dirty_and_untracked(self):
        (self.repo / 'source').write_text('dirty')
        self.refused()
        self.git(self.repo, 'checkout', '--', 'source')
        (self.repo / 'untracked').write_text('unique')
        self.refused()

    def test_hidden_tracked_changes(self):
        self.git(self.repo, 'update-index', '--assume-unchanged', 'source')
        (self.repo / 'source').write_text('hidden changes')
        self.refused()

    def test_ignored_runtime(self):
        (self.repo / 'runtime').mkdir()
        (self.repo / 'runtime/private').write_text('unique')
        self.refused()

    def test_detached_commit(self):
        self.git(self.repo, 'checkout', '--detach')
        self.commit_unique()
        self.refused()

    def test_snapshot_and_tag(self):
        self.git(self.repo, 'checkout', '--detach')
        self.commit_unique()
        self.git(self.repo, 'update-ref', 'refs/wip/test', 'HEAD')
        self.git(self.repo, 'tag', '-a', 'private-tag', '-m', 'private tag')
        self.git(self.repo, 'checkout', self.branch)
        self.refused()
        self.git(self.repo, 'update-ref', '-d', 'refs/wip/test')
        self.refused()

    def test_stash(self):
        (self.repo / 'source').write_text('stash')
        self.git(self.repo, 'stash', 'push')
        self.refused()

    def test_other_remote_is_insufficient(self):
        other = self.root / 'other.git'
        self.git(self.root, 'init', '--bare', str(other))
        self.commit_unique()
        self.git(self.repo, 'remote', 'add', 'other', str(other))
        self.git(self.repo, 'push', 'other', 'HEAD')
        self.refused()

    def test_stale_tracking_refs(self):
        self.commit_unique()
        self.git(self.repo, 'push', 'origin', 'HEAD:temporary')
        self.git(self.remote, 'update-ref', '-d', 'refs/heads/temporary')
        self.refused()

    def test_missing_origin_and_git_error(self):
        self.git(self.repo, 'remote', 'set-url', 'origin', str(self.root / 'missing'))
        self.refused()
        (self.repo / '.git/index').write_bytes(b'broken index')
        self.refused()

    def test_worktree_blocks_parent_and_preserves_metadata(self):
        wt = self.root / 'worktree'
        self.git(self.repo, 'worktree', 'add', '--detach', str(wt))
        self.refused()
        self.assertEqual(safety.check(wt, worktree=True), str(self.remote))
        self.git(wt, 'config', 'user.name', 'Cleanup Test')
        (wt / 'source').write_text('detached unique')
        self.git(wt, 'commit', '-am', 'unique')
        with self.assertRaises(ValueError):
            safety.check(wt, worktree=True)
        self.assertTrue(wt.exists())

    def test_submodule(self):
        self.git(self.repo, '-c', 'protocol.file.allow=always', 'submodule', 'add', str(self.remote), 'sub')
        self.git(self.repo, 'commit', '-am', 'submodule')
        self.git(self.repo, 'push', 'origin', 'HEAD')
        self.refused()

    def test_worktree_trash_move_keeps_git_usable_and_locked(self):
        wt = self.root / 'worktree'
        self.git(self.repo, 'worktree', 'add', '--detach', str(wt))
        home = self.root / 'fake-home'
        (home / '.Trash').mkdir(parents=True)
        with patch.object(Path, 'home', return_value=home):
            safety.move_worktree(str(self.repo), str(wt))
        self.assertFalse(wt.exists())
        moved = next((home / '.Trash').glob('*/worktree'))
        self.assertEqual(self.git(moved, 'show', 'HEAD:source'), 'original')
        self.assertIn('locked cleanup:', self.git(self.repo, 'worktree', 'list', '--porcelain'))
        self.git(self.repo, 'worktree', 'prune', '--expire', 'now')
        self.assertEqual(self.git(moved, 'status', '--porcelain'), '')
        self.refused()

    def test_cargo_target_rejects_tracked_and_symlink_paths(self):
        target = self.repo / 'target'
        target.mkdir()
        (target / 'CACHEDIR.TAG').write_text('created by cargo')
        safety.cargo_target(target)
        (target / 'precious').write_text('tracked')
        self.git(self.repo, 'add', 'target/precious')
        with self.assertRaises(ValueError):
            safety.cargo_target(target)
        alias = self.root / 'alias'
        alias.symlink_to(self.repo, target_is_directory=True)
        with self.assertRaises(ValueError):
            safety.cargo_target(alias / 'target')

    def test_copy_equal_size_collision_and_symlinks(self):
        a, b = self.root / 'a', self.root / 'b'
        a.mkdir(); b.mkdir()
        (a / 'file').write_text('aaaa'); (b / 'file').write_text('bbbb')
        self.assertFalse(safety.verify(a, b))
        (b / 'file').write_text('aaaa')
        self.assertTrue(safety.verify(a, b))
        (a / 'link').symlink_to('file'); (b / 'link').symlink_to('wrong')
        self.assertFalse(safety.verify(a, b))
        (b / 'link').unlink(); (b / 'link').symlink_to('file')
        self.assertTrue(safety.verify(a, b))
        (b / 'extra').write_text('')
        self.assertFalse(safety.verify(a, b))


if __name__ == '__main__':
    unittest.main()
