#!/usr/bin/env python3
"""Loopback-only protocol checks; never scan the LAN."""
import asyncio
import importlib.machinery
import importlib.util
import ipaddress
from unittest.mock import patch
from pathlib import Path
import ssl
import subprocess
import tempfile
import unittest

loader = importlib.machinery.SourceFileLoader('lan_http', str(Path(__file__).resolve().parents[2] / 'bin/lan-http'))
spec = importlib.util.spec_from_loader(loader.name, loader)
scanner = importlib.util.module_from_spec(spec)
loader.exec_module(scanner)


class NetworkTests(unittest.TestCase):
    CONFIG = """lo0: flags=8049<UP,LOOPBACK,RUNNING>
    inet 127.0.0.1 netmask 0xff000000
en0: flags=8863<UP,BROADCAST,RUNNING>
    inet 192.168.1.20 netmask 0xffffff00
    status: active
en1: flags=8863<UP,BROADCAST,RUNNING>
    inet 172.16.2.2 netmask 0xffffff00
    status: active
en2: flags=8863<UP,BROADCAST,RUNNING>
    inet 10.2.3.4 netmask 0xffffff00
    status: inactive
utun1: flags=8051<UP,POINTOPOINT,RUNNING>
    inet 100.64.1.2 --> 100.64.1.2 netmask 0xffffffff
"""
    ROUTES = """Destination Gateway Flags Netif Expire
default 192.168.1.1 UGScg en0
192.168.1 link#14 UCS en0
192.168.1.1 aa:bb:cc:dd:ee:ff UHLWI en0
192.168.1.255 ff:ff:ff:ff:ff:ff UHLWbI en0
192.168.8/24 192.168.1.1 UGSc en0
10.42/16 172.16.2.1 UGSc en1
10.99/16 172.16.2.1 UGR en1
127 127.0.0.1 UCS lo0
224.0.0/4 link#14 UmCS en0
100.64/10 link#30 UCS utun1
0/1 link#30 UCS utun1
"""

    def test_interfaces_routes_and_home_range(self):
        with patch.object(scanner.subprocess, 'check_output', side_effect=[self.CONFIG, self.ROUTES]):
            self.assertEqual(set(map(str, scanner.local_networks())),
                             {'192.168.1.0/24', '192.168.8.0/24', '172.16.2.0/24', '10.42.0.0/16', '100.64.1.2/32'})
        with patch.object(scanner.subprocess, 'check_output', return_value=self.CONFIG):
            self.assertEqual(list(map(str, scanner.local_networks('en0'))), ['192.168.1.0/24'])
        routes = set(map(str, scanner.route_networks(self.ROUTES)))
        self.assertIn('192.168.8.0/24', routes)
        self.assertNotIn('10.99.0.0/16', routes)
        self.assertNotIn('192.168.1.255/32', routes)

    def test_adjacent_subnets_keep_host_boundaries(self):
        networks = list(map(ipaddress.IPv4Network, ['10.1.0.0/24', '10.1.1.0/24', '10.1.1.2/32']))
        self.assertEqual(scanner.unique_networks(networks), networks[:2])

    def test_title_entities_case_and_controls(self):
        self.assertEqual(scanner.page_title(b'<TITLE> Router &amp;  Switch\n Control\x1b </TITLE>'),
                         'Router & Switch Control')
        self.assertEqual(scanner.page_title(b'<meta charset="windows-1252"><title>Caf\xe9</title>'), 'Café')
        self.assertEqual(scanner.page_title(b'<html>No title</html>'), '')


class DiscoveryTests(unittest.IsolatedAsyncioTestCase):
    async def test_router_discovery_is_bounded_and_keeps_known_subnets(self):
        known = list(map(ipaddress.IPv4Network, ['192.168.1.0/24', '10.42.0.0/24']))
        calls = []
        active = 0
        peak = 0

        class Writer:
            def close(self):
                pass

            async def wait_closed(self):
                pass

        async def connect(host, port):
            nonlocal active, peak
            calls.append((host, port))
            active += 1
            peak = max(peak, active)
            await asyncio.sleep(0)
            active -= 1
            if (host, port) in [('192.168.8.1', 80), ('192.168.9.254', 443)]:
                return None, Writer()
            raise ConnectionRefusedError

        with patch.object(scanner.asyncio, 'open_connection', side_effect=connect):
            result = await scanner.discover_networks(known, 16, 0.1)
        self.assertEqual(set(map(str, result)),
                         {'192.168.1.0/24', '10.42.0.0/24', '192.168.8.0/24', '192.168.9.0/24'})
        self.assertLessEqual(peak, 16)
        self.assertLessEqual(len(calls), 255 * 4)
        self.assertFalse(any(host.startswith('192.168.1.') for host, _ in calls))
        self.assertTrue(all(host.rsplit('.', 1)[1] in ('1', '254') and port in (80, 443)
                            for host, port in calls))

    async def test_title_redirect_chunking_and_size_limit(self):
        async def handler(reader, writer):
            try:
                request = await reader.readuntil(b'\r\n\r\n')
                path = request.split()[1]
                if path == b'/':
                    writer.write(b'HTTP/1.1 302 Found\r\nLocation: /home\r\nContent-Length: 0\r\n\r\n')
                elif path == b'/home':
                    writer.write(b'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n')
                    for chunk in [b'<html><title>LAN ', b'&amp; Admin</title></html>']:
                        writer.write(f'{len(chunk):x}\r\n'.encode() + chunk + b'\r\n')
                    writer.write(b'0\r\n\r\n')
                elif path == b'/large':
                    writer.write(b'HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n' + b'x' * 150000 + b'<title>Too late</title>')
                elif path == b'/slow':
                    writer.write(b'HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n')
                    await writer.drain()
                    await reader.read()
                await writer.drain()
            except (OSError, asyncio.IncompleteReadError):
                pass
            finally:
                writer.close()
        server = await asyncio.start_server(handler, '127.0.0.1', 0)
        async with server:
            url = f'http://127.0.0.1:{server.sockets[0].getsockname()[1]}'
            self.assertEqual(await scanner.fetch_title(url + '/'), 'LAN & Admin')
            self.assertEqual(await scanner.fetch_title(url + '/large'), '')
            self.assertEqual(await scanner.fetch_title(url + '/slow', timeout=0.1), '')

    async def test_http_https_and_non_http(self):
        # Protocol detection deliberately makes incomplete/wrong-protocol TLS connections.
        asyncio.get_running_loop().set_debug(False)
        async def http(reader, writer):
            try:
                data = await reader.read(8192)
                if data.startswith(b'HEAD '):
                    writer.write(b'HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n')
                    await writer.drain()
            finally:
                writer.close()

        async def other(reader, writer):
            writer.write(b'SSH-2.0-test\r\n')
            await writer.drain()
            writer.close()

        async def silent(reader, writer):
            await reader.read()
            writer.close()

        with tempfile.TemporaryDirectory() as directory:
            cert, key = f'{directory}/cert.pem', f'{directory}/key.pem'
            subprocess.run(['openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes',
                            '-keyout', key, '-out', cert, '-days', '1', '-subj', '/CN=localhost'],
                           check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            tls.load_cert_chain(cert, key)
            client = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
            client.check_hostname = False
            client.verify_mode = ssl.CERT_NONE
            servers = []
            try:
                for handler, context in [(http, None), (http, tls), (other, None), (silent, None)]:
                    servers.append(await asyncio.start_server(handler, '127.0.0.1', 0, ssl=context))
                ports = [server.sockets[0].getsockname()[1] for server in servers]
                results = await asyncio.gather(*(scanner.probe('127.0.0.1', port, 0.3, client) for port in ports))
                self.assertEqual(results[0], (f'http://127.0.0.1:{ports[0]}/', '401'))
                self.assertEqual(results[1], (f'https://127.0.0.1:{ports[1]}/', '401'))
                self.assertEqual(await scanner.fetch_title(results[1][0]), '')
                self.assertIsNone(results[2])
                self.assertIsNone(results[3])
            finally:
                for server in servers:
                    server.close()
                    await server.wait_closed()
            self.assertIsNone(await scanner.probe('127.0.0.1', ports[0], 0.3, client))


if __name__ == '__main__':
    unittest.main()
