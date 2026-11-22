#!/usr/bin/env python3
"""Loopback-only protocol checks; never scan the LAN."""
import asyncio
import importlib.machinery
import importlib.util
from pathlib import Path
import ssl
import subprocess
import tempfile
import unittest

loader = importlib.machinery.SourceFileLoader('lan_http', str(Path(__file__).resolve().parents[2] / 'bin/lan-http'))
spec = importlib.util.spec_from_loader(loader.name, loader)
scanner = importlib.util.module_from_spec(spec)
loader.exec_module(scanner)


class DiscoveryTests(unittest.IsolatedAsyncioTestCase):
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
                self.assertIsNone(results[2])
                self.assertIsNone(results[3])
            finally:
                for server in servers:
                    server.close()
                    await server.wait_closed()
            self.assertIsNone(await scanner.probe('127.0.0.1', ports[0], 0.3, client))


if __name__ == '__main__':
    unittest.main()
