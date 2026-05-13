import { describe, expect, it } from 'vitest';
import { spawn } from 'node:child_process';
import * as path from 'node:path';

function frame(payload: object): string {
  const body = JSON.stringify(payload);
  return `Content-Length: ${Buffer.byteLength(body, 'utf8')}\r\n\r\n${body}`;
}

describe('bin/sui-lsp.js stdio entry', () => {
  it('responds to initialize when launched via the bin wrapper', async () => {
    const bin = path.join(__dirname, '..', 'bin', 'sui-lsp.js');
    const child = spawn('node', [bin], { stdio: ['pipe', 'pipe', 'pipe'] });
    let buf = Buffer.alloc(0);
    child.stdout.on('data', (b: Buffer) => { buf = Buffer.concat([buf, b]); });
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
