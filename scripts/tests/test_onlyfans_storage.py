# /// script
# requires-python = ">=3.12,<3.13"
# dependencies = ["plyvel-ci==1.5.1"]
# ///
"""Synthetic LevelDB checks; never accesses browser data."""
from pathlib import Path
import runpy
import tempfile
import unittest
import plyvel

READ = runpy.run_path(str(Path(__file__).resolve().parents[2] / 'bin/onlyfans-browser-key'))['read_token']
KEY = b'_https://onlyfans.com\x00\x01bcTokenSha'


class StorageTests(unittest.TestCase):
    def test_latest_value_and_deletion_while_database_open(self):
        with tempfile.TemporaryDirectory() as temp:
            dbpath = Path(temp) / 'Local Storage/leveldb'
            dbpath.parent.mkdir()
            with plyvel.DB(str(dbpath), create_if_missing=True) as db:
                db.put(KEY, b'\x01old', sync=True)
                db.put(KEY, b'\x01new', sync=True)
                self.assertEqual(READ(temp), 'new')
                db.delete(KEY, sync=True)
                with self.assertRaises(ValueError):
                    READ(temp)

    def test_compacted_utf16_and_wrong_origin(self):
        with tempfile.TemporaryDirectory() as temp:
            dbpath = Path(temp) / 'Local Storage/leveldb'
            dbpath.parent.mkdir()
            with plyvel.DB(str(dbpath), create_if_missing=True) as db:
                db.put(KEY, b'\x00' + '"fixture"'.encode('utf-16-le'), sync=True)
                db.put(b'_https://example.com\x00\x01bcTokenSha', b'\x01wrong', sync=True)
                db.compact_range()
                self.assertEqual(READ(temp), 'fixture')


if __name__ == '__main__':
    unittest.main()
