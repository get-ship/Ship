import { describe, expect, it } from 'vitest';
import { spawn } from 'node:child_process';
import * as path from 'node:path';

function frame(payload: object): string {
  const body = JSON.stringify(payload);
  return `Content-Length: ${Buffer.byteLength(body, 'utf8')}\r\n\r\n${body}`;
}

describe('server smoke', () => {
  it('responds to initialize over stdio', async () => {
    const dist = path.join(__dirname, '..', 'dist', 'server.js');
    const child = spawn('node', [dist, '--stdio'], { stdio: ['pipe', 'pipe', 'pipe'] });
    let buf = Buffer.alloc(0);
    const onChunk = (b: Buffer) => { buf = Buffer.concat([buf, b]); };
    child.stdout.on('data', onChunk);
    child.stderr.on('data', () => { /* swallow */ });

    const init = frame({
      jsonrpc: '2.0', id: 1, method: 'initialize',
      params: { processId: process.pid, rootUri: null, capabilities: {} },
    });
    child.stdin.write(init);

    const result = await new Promise<string>((resolve, reject) => {
      const timer = setTimeout(() => { reject(new Error('timeout')); }, 5000);
      const tick = () => {
        const text = buf.toString('utf8');
        if (text.includes('"capabilities"')) { clearTimeout(timer); resolve(text); return; }
        setTimeout(tick, 25);
      };
      tick();
    });
    expect(result).toContain('"capabilities"');
    expect(result).toContain('completionProvider');
    expect(result).toContain('hoverProvider');
    child.kill();
  }, 15000);
});
