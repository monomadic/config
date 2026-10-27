#!/usr/bin/env python3
"""Fail-closed checks used by cleanup. No media or checkout deletion here."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def git(repo, *args):
    result = subprocess.run(['git', '-C', str(repo), *args], stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, env={**os.environ, 'GIT_TERMINAL_PROMPT': '0'})
    if result.returncode:
        raise ValueError('Git inspection failed: ' + result.stderr.decode(errors='replace').strip())
    return result.stdout


def check(repo, worktree=False):
    repo = Path(repo)
    if repo.is_symlink() or git(repo, 'rev-parse', '--show-toplevel').decode().strip() != str(repo.resolve()):
        raise ValueError('not a standalone checkout path')
    if git(repo, 'status', '--porcelain=v1', '-z', '--untracked-files=all', '--ignored=matching', '--ignore-submodules=none'):
        raise ValueError('uncommitted, untracked or ignored files exist; clean build caches separately')
    if any(record[:1].islower() or record.startswith(b'S ')
           for record in git(repo, 'ls-files', '-v', '-z').split(b'\0') if record):
        raise ValueError('assume-unchanged or skip-worktree entries require manual review')
    if git(repo, 'ls-files', '--stage').find(b'160000 ') >= 0:
        raise ValueError('submodules require manual review')
    common = Path(git(repo, 'rev-parse', '--path-format=absolute', '--git-common-dir').decode().strip())
    if not worktree:
        if not (repo / '.git').is_dir() or common.resolve() != (repo / '.git').resolve():
            raise ValueError('linked or external Git metadata requires manual review')
        records = git(repo, 'worktree', 'list', '--porcelain').splitlines()
        if sum(line.startswith(b'worktree ') for line in records) != 1:
            raise ValueError('other worktrees exist, including worktrees in Trash')
    if git(repo, 'stash', 'list'):
        raise ValueError('stashes exist')
    initial_refs = git(repo, 'show-ref', '--head')
    remote = git(repo, 'remote', 'get-url', 'origin').decode().strip()
    # A separate clone neither trusts stale tracking refs nor changes local refs.
    with tempfile.TemporaryDirectory(prefix='cleanup-remote-') as tmp:
        mirror = Path(tmp) / 'mirror.git'
        result = subprocess.run(['git', 'clone', '--mirror', '--no-local', '--', remote, str(mirror)],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=repo, timeout=120,
                                env={**os.environ, 'GIT_TERMINAL_PROMPT': '0'})
        if result.returncode:
            raise ValueError('cannot verify recovery from origin: ' + result.stderr.decode(errors='replace').strip())
        refs = git(repo, 'for-each-ref', '--format=%(refname)').decode().splitlines()
        refs = [r for r in refs if not r.startswith('refs/remotes/')]
        # Include every object reachable from HEAD and local refs, including tags and snapshots.
        objects = git(repo, 'rev-list', '--objects', '--no-object-names', 'HEAD', *refs).splitlines()
        result = subprocess.run(['git', '-C', str(mirror), 'cat-file', '--batch-check=%(objectname) %(objecttype)'],
                                input=b'\n'.join(objects)+b'\n', stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if result.returncode or any(line.endswith(b' missing') for line in result.stdout.splitlines()):
            raise ValueError('HEAD or local refs contain objects unavailable from origin')
    if git(repo, 'show-ref', '--head') != initial_refs or git(
            repo, 'status', '--porcelain=v1', '-z', '--untracked-files=all',
            '--ignored=matching', '--ignore-submodules=none'):
        raise ValueError('checkout changed during verification; source kept')
    return remote


def verify(source, destination):
    """Compare names, types, symlink targets and regular-file bytes; reject special files."""
    a, b = Path(source), Path(destination)
    if a.is_symlink():
        return b.is_symlink() and os.readlink(a) == os.readlink(b)
    if a.is_file():
        if b.is_symlink() or not b.is_file() or a.stat().st_size != b.stat().st_size:
            return False
        with a.open('rb') as x, b.open('rb') as y:
            while True:
                chunk = x.read(1024 * 1024)
                if chunk != y.read(1024 * 1024):
                    return False
                if not chunk:
                    return True
    if a.is_dir():
        if b.is_symlink() or not b.is_dir():
            return False
        names = {p.name for p in a.iterdir()}
        return names == {p.name for p in b.iterdir()} and all(verify(a/n, b/n) for n in names)
    return False


def cargo_target(target):
    target = Path(target)
    if target.name != 'target' or str(target.resolve()) != str(target):
        raise ValueError('cargo target path is not canonical')
    marker = target / 'CACHEDIR.TAG'
    if marker.is_symlink() or not marker.is_file() or 'created by cargo' not in marker.read_text():
        raise ValueError('cargo cache marker missing')
    for directory, dirs, files in os.walk(target, followlinks=False):
        if '.git' in dirs or '.git' in files:
            raise ValueError('cargo target contains a nested repository')
    # A cache marker must not authorize deleting versioned source files.
    result = subprocess.run(['git', '-C', str(target.parent), 'rev-parse', '--show-toplevel'],
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if result.returncode == 0:
        root = Path(result.stdout.decode().strip())
        relative = target.relative_to(root)
        if git(root, 'ls-files', '-z', '--', str(relative)):
            raise ValueError('cargo target contains tracked files')
    elif any((parent / '.git').exists() for parent in target.parents):
        raise ValueError('cannot inspect cargo project Git metadata')


def move_worktree(repo, wt):
    check(wt, worktree=True)
    common = git(repo, 'rev-parse', '--path-format=absolute', '--git-common-dir')
    if common != git(wt, 'rev-parse', '--path-format=absolute', '--git-common-dir'):
        raise ValueError('worktree belongs to another repository')
    trash = Path.home() / '.Trash'
    if not trash.is_dir():
        raise ValueError('Trash is unavailable')
    holder = Path(tempfile.mkdtemp(prefix='cleanup-worktree-', dir=trash))
    dest = holder / Path(wt).name
    try:
        git(repo, 'worktree', 'move', '--', wt, str(dest))
        try:
            git(repo, 'worktree', 'lock', '--reason', 'cleanup: retained in Trash for recovery', str(dest))
        except ValueError:
            git(repo, 'worktree', 'move', '--', str(dest), wt)
            raise
    finally:
        if not any(holder.iterdir()):
            holder.rmdir()
    print('Worktree retained at ' + str(dest) + '; its Git metadata remains in the main repository.')


if __name__ == '__main__':
    try:
        action, *args = sys.argv[1:]
        if action == 'check':
            print(check(*args))
        elif action == 'cargo':
            cargo_target(*args)
        elif action == 'worktree':
            move_worktree(*args)
        elif action == 'verify':
            if not verify(*args):
                raise ValueError('copy contents differ; source kept')
        else:
            raise ValueError('unknown safety operation')
    except (ValueError, OSError, subprocess.TimeoutExpired) as exc:
        print(str(exc), file=sys.stderr)
        sys.exit(1)
