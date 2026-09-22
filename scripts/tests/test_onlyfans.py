"""Offline regression checks; use the Python environment containing yt-dlp."""
import hashlib
import json
import os
from pathlib import Path
import runpy
import socket
import unittest
from unittest.mock import patch
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[2]
IE = runpy.run_path(str(ROOT / 'config/yt-dlp/plugins/faphouse/yt_dlp_plugins/extractor/onlyfans.py'))['OnlyFansIE']
RULES = json.loads((ROOT / 'bin/lib/onlyfans-rules-candidate.json').read_text())
CHECK = runpy.run_path(str(ROOT / 'bin/onlyfans-signature-check.py'))


class OnlyFansOffline(unittest.TestCase):
    def setUp(self):
        self.network = patch.object(socket.socket, 'connect', side_effect=AssertionError('Network forbidden'))
        self.network.start()
        self.addCleanup(self.network.stop)
        self.env = patch.dict(os.environ, {}, clear=True)
        self.env.start()
        self.addCleanup(self.env.stop)
        self.ie = IE()
        self.ie._get_cookies = lambda _: {'auth_id': SimpleNamespace(value='123')}
        self.ie._download_webpage = lambda *a, **kw: self.fail('Unexpected CDN request')

    def identity(self):
        os.environ.update(YT_DLP_ONLYFANS_UNSAFE_API='1',
                          YT_DLP_ONLYFANS_USER_AGENT='Offline browser fixture',
                          YT_DLP_ONLYFANS_X_BC='offline-bc')

    def test_guard_precedes_cookies_and_network(self):
        self.ie._get_cookies = lambda _: self.fail('Cookies accessed before guard')
        with self.assertRaisesRegex(Exception, 'disabled'):
            self.ie._real_initialize()
        with self.assertRaisesRegex(Exception, 'disabled'):
            self.ie._download_api_json('/posts/1', '1', 'test', 'https://onlyfans.com/')

    def test_identity_required_and_header_injection_rejected(self):
        os.environ['YT_DLP_ONLYFANS_UNSAFE_API'] = '1'
        self.ie._get_cookies = lambda _: self.fail('Cookies accessed before identity validation')
        with self.assertRaisesRegex(Exception, 'USER_AGENT'):
            self.ie._real_initialize()
        self.identity()
        os.environ['YT_DLP_ONLYFANS_X_BC'] = 'bad\r\nheader'
        with self.assertRaisesRegex(Exception, 'X_BC'):
            self.ie._real_initialize()

    def test_headers_and_signature_match_recovered_rules(self):
        self.identity()
        os.environ['YT_DLP_ONLYFANS_X_HASH'] = 'offline-hash'
        self.ie._real_initialize()
        path = '/api2/v2/posts/1?skip_users=all&test=a%2Bb'
        with patch('time.time', return_value=1700000000):
            headers = self.ie._signed_headers(path, 'https://onlyfans.com/1/example')
        digest = hashlib.sha1(f'{RULES["static_param"]}\n1700000000000\n{path}\n123'.encode()).hexdigest()
        checksum = abs(RULES['checksum_constant'] + sum(ord(digest[i]) for i in RULES['checksum_indexes']))
        self.assertEqual(headers['sign'], RULES['format'].format(digest, checksum))
        self.assertEqual(headers['x-of-rev'], RULES['revision'])
        self.assertEqual(headers['User-Agent'], 'Offline browser fixture')
        self.assertEqual(headers['x-bc'], 'offline-bc')
        self.assertEqual(headers['x-hash'], 'offline-hash')
        constants = CHECK['load_constants']()
        results = CHECK['compare'](constants, 'https://onlyfans.com'+path, headers['time'], '123', headers['sign'], headers['x-of-rev'], headers['User-Agent'])
        self.assertIsNone(results.pop('User-Agent'))
        self.assertTrue(all(results.values()))

    def test_automatic_key_and_environment_override(self):
        self.identity()
        with patch.object(self.ie, '_stored_browser_key', return_value='stored-fixture') as read:
            self.ie._real_initialize()
            read.assert_not_called()
            del os.environ['YT_DLP_ONLYFANS_X_BC']
            self.ie._real_initialize()
            read.assert_called_once()
            self.assertEqual(self.ie._bc_token, 'stored-fixture')

    def test_automatic_user_agent_and_override(self):
        self.identity()
        with patch.object(self.ie, '_installed_browser_user_agent', return_value='automatic-fixture') as read:
            self.ie._real_initialize()
            read.assert_not_called()
            del os.environ['YT_DLP_ONLYFANS_USER_AGENT']
            self.ie._real_initialize()
            self.assertEqual(self.ie._browser_user_agent, 'automatic-fixture')
            read.assert_called_once()

    def test_reduced_user_agent_and_ambiguous_install(self):
        import plistlib
        from yt_dlp import YoutubeDL
        ie = IE(YoutubeDL({'cookiesfrombrowser': ('brave',)}))
        def plist(major):
            return plistlib.dumps({'CFBundleIdentifier': 'com.brave.Browser',
                                   'CFBundleShortVersionString': f'{major}.1.95.104'})
        with patch('sys.platform', 'darwin'), patch.object(Path, 'exists', return_value=True):
            with patch.object(Path, 'read_bytes', return_value=plist(153)):
                self.assertEqual(ie._installed_browser_user_agent(),
                    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 '
                    '(KHTML, like Gecko) Chrome/153.0.0.0 Safari/537.36')
            with patch.object(Path, 'read_bytes', side_effect=[plist(153), plist(154)]):
                with self.assertRaisesRegex(Exception, 'ambiguous'):
                    ie._installed_browser_user_agent()
            with patch.object(Path, 'read_bytes', return_value=plist(1)):
                with self.assertRaisesRegex(Exception, 'installed Brave version'):
                    ie._installed_browser_user_agent()

    def test_selected_cookie_profile_is_used(self):
        from tempfile import TemporaryDirectory
        from yt_dlp import YoutubeDL
        with TemporaryDirectory() as temp:
            profile = Path(temp) / 'Profile 2'
            (profile / 'Network').mkdir(parents=True)
            (profile / 'Network/Cookies').touch()
            ie = IE(YoutubeDL({'cookiesfrombrowser': ('brave', str(profile))}))
            with patch('subprocess.run', return_value=SimpleNamespace(stdout='fixture-key\n')) as run:
                self.assertEqual(ie._stored_browser_key(), 'fixture-key')
                self.assertEqual(run.call_args.args[0][1], str(profile))

    def test_optional_hash_and_missing_auth_cookie(self):
        self.identity()
        self.ie._real_initialize()
        self.assertNotIn('x-hash', self.ie._signed_headers('/api2/v2/posts/1', 'https://onlyfans.com/'))
        self.ie._get_cookies = lambda _: {}
        with self.assertRaisesRegex(Exception, 'auth_id'):
            self.ie._real_initialize()


if __name__ == '__main__':
    unittest.main()
