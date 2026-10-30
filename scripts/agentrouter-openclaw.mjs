#!/usr/bin/env node
// Local, authenticated OpenClaw gateway for HeraMind. Credentials live only
// under the git-ignored data directory or in the caller's environment.
import { readFile, writeFile, mkdir, access } from 'node:fs/promises'
import { spawn } from 'node:child_process'
import { dirname, join, delimiter, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = dirname(dirname(fileURLToPath(import.meta.url)))
const data = resolve(process.env.HERAMIND_AGENTROUTER_DATA_DIR || join(root, 'data/agentrouter-openclaw'))
const command = process.argv[2] || 'start'
if (!['start', 'status', 'test'].includes(command)) {
  console.error('Usage: node scripts/agentrouter-openclaw.mjs [start|status|test]')
  process.exit(1)
}

let credentials = {}
try { credentials = JSON.parse(await readFile(join(data, 'secrets.json'), 'utf8')) } catch (error) {
  if (error.code !== 'ENOENT') throw error
}
const apiKey = process.env.HERAMIND_AGENTROUTER_KEY || credentials.apiKey
const gatewayToken = process.env.HERAMIND_OPENCLAW_GATEWAY_TOKEN || credentials.gatewayToken
const redact = value => [apiKey, gatewayToken].filter(Boolean).reduce(
  (text, secret) => text.replaceAll(secret, '[REDACTED]'), String(value)
)
if (!gatewayToken || (command === 'start' && !apiKey)) {
  console.error('Missing bridge credentials. See docs/agentrouter.md for setup.')
  process.exit(1)
}

if (command === 'start') {
  const node = join(data, process.platform === 'win32' ? 'node-runtime/node.exe' : 'node-runtime/bin/node')
  const cli = join(data, 'client/node_modules/openclaw/openclaw.mjs')
  try { await Promise.all([access(node), access(cli)]) } catch {
    console.error('OpenClaw runtime is missing. See docs/agentrouter.md for installation.')
    process.exit(1)
  }
  await mkdir(join(data, 'workspace'), { recursive: true, mode: 0o700 })
  const runtimeConfig = join(data, 'openclaw.json')
  await writeFile(runtimeConfig, await readFile(process.env.HERAMIND_OPENCLAW_CONFIG_TEMPLATE || join(root, 'deploy/agentrouter-openclaw.config.json')), { mode: 0o600 })
  const child = spawn(node, [cli, 'gateway', 'run', '--port', '19376', '--bind', 'loopback'], {
    cwd: join(data, 'workspace'),
    env: {
      PATH: [dirname(node), process.env.PATH || ''].join(delimiter),
      TZ: process.env.TZ || 'Asia/Ho_Chi_Minh',
      NO_COLOR: '1',
      OPENCLAW_CONFIG_PATH: runtimeConfig,
      OPENCLAW_STATE_DIR: join(data, 'state'),
      HERAMIND_AGENTROUTER_WORKSPACE: join(data, 'workspace'),
      HERAMIND_AGENTROUTER_KEY: apiKey,
      HERAMIND_OPENCLAW_GATEWAY_TOKEN: gatewayToken,
    },
    stdio: ['ignore', 'pipe', 'pipe'],
  })
  // Redact whole lines, including secrets split across pipe chunks.
  for (const [stream, output] of [[child.stdout, process.stdout], [child.stderr, process.stderr]]) {
    let pending = ''
    stream.setEncoding('utf8')
    stream.on('data', chunk => {
      pending += chunk
      let newline
      while ((newline = pending.indexOf('\n')) !== -1) {
        output.write(`${redact(pending.slice(0, newline))}\n`)
        pending = pending.slice(newline + 1)
      }
    })
    stream.on('end', () => { if (pending) output.write(redact(pending)) })
  }
  for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => child.kill(signal))
  // When supervised by HeraMind, stop the gateway if its owning server exits,
  // including a server terminated before its normal shutdown hooks run.
  let parentWatch
  if (process.env.HERAMIND_BRIDGE_PARENT_PID) {
    const parent = Number(process.env.HERAMIND_BRIDGE_PARENT_PID)
    if (!Number.isSafeInteger(parent) || parent <= 1) {
      child.kill('SIGTERM')
      throw new Error('Invalid supervising server PID')
    }
    parentWatch = setInterval(() => {
      try { process.kill(parent, 0) } catch { child.kill('SIGTERM') }
    }, 2000)
    parentWatch.unref()
  }
  child.on('error', error => { console.error(redact(error.message)); process.exitCode = 1 })
  child.on('exit', (code, signal) => { if (parentWatch) clearInterval(parentWatch); process.exitCode = code ?? (signal === 'SIGINT' ? 130 : 1) })
} else {
  try {
    const response = await fetch(`http://127.0.0.1:19376/v1/${command === 'test' ? 'chat/completions' : 'models'}`, {
      method: command === 'test' ? 'POST' : 'GET',
      headers: { Authorization: `Bearer ${gatewayToken}`, 'Content-Type': 'application/json' },
      ...(command === 'test' ? { body: JSON.stringify({ model: 'openclaw/default', messages: [{ role: 'user', content: 'Reply with only the word OK.' }], max_tokens: 256 }) } : {}),
      signal: AbortSignal.timeout(60000),
      redirect: 'error',
    })
    const body = await response.json()
    if (!response.ok) throw new Error(`HTTP ${response.status}: ${JSON.stringify(body)}`)
    console.log(redact(JSON.stringify(command === 'test'
      ? { status: response.status, response: body.choices?.[0]?.message?.content }
      : { status: response.status, models: body.data?.map(model => model.id) }, null, 2)))
    if (command === 'test' && !body.choices?.[0]?.message?.content?.trim()) process.exitCode = 1
  } catch (error) {
    console.error(redact(error.message))
    process.exitCode = 1
  }
}
