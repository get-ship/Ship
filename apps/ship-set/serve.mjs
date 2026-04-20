import http from 'node:http';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, 'dist');
const port = Number(process.env.PORT ?? 4322);
const host = process.env.HOST ?? '0.0.0.0';

const mime = {
  '.html': 'text/html; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.ico': 'image/x-icon',
  '.woff2': 'font/woff2',
};

const send = (res, status, body, type = 'text/plain; charset=utf-8') => {
  res.writeHead(status, { 'content-type': type, 'cache-control': 'no-cache' });
  res.end(body);
};

const resolveFile = async (urlPath) => {
  const clean = decodeURIComponent(urlPath.split('?')[0]);
  const safe = path.normalize(path.join(root, clean)).replace(/\\/g, '/');
  if (!safe.startsWith(root)) return null;
  for (const c of [safe, path.join(safe, 'index.html'), `${safe}.html`]) {
    try { const st = await fs.stat(c); if (st.isFile()) return c; } catch {}
  }
  return null;
};

http
  .createServer(async (req, res) => {
    const file = await resolveFile(req.url ?? '/');
    if (!file) return send(res, 404, 'not found');
    const ext = path.extname(file);
    const body = await fs.readFile(file);
    send(res, 200, body, mime[ext] ?? 'application/octet-stream');
  })
  .listen(port, host, () => {
    console.log(`ship-set preview on http://${host}:${port}`);
  });
