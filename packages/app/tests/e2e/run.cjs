const { spawn } = require('node:child_process')
const { mkdirSync } = require('node:fs')
const { tmpdir } = require('node:os')
const { dirname, join } = require('node:path')

const scriptDir = __dirname
// tests/e2e -> app (two levels up), then repo root (three levels up).
const appDir = join(scriptDir, '..', '..')
const repoRoot = join(scriptDir, '..', '..', '..')

const e2eDir = join(tmpdir(), `logholizon-e2e-${process.pid}`)
const e2eDb = join(e2eDir, 'core.db').replace(/\\/g, '/')
const corePort = 8788
const appPort = 3100

mkdirSync(e2eDir, { recursive: true })

function run(command, args, options) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { ...options, stdio: 'inherit', shell: false })
    child.on('error', reject)
    child.on('exit', (code) => {
      if (code === 0) resolve()
      else reject(new Error(`${command} ${args.join(' ')} exited with code ${code}`))
    })
  })
}

async function waitFor(url, timeoutMs) {
  const start = Date.now()
  while (Date.now() - start < timeoutMs) {
    try {
      const response = await fetch(url)
      if (response.ok) return
    } catch {
      // Server not up yet.
    }
    await new Promise((resolve) => setTimeout(resolve, 1000))
  }
  throw new Error(`timed out waiting for ${url}`)
}

async function main() {
  const dbUrl = `sqlite://${e2eDb}?mode=rwc`
  const nodeDir = dirname(process.execPath)
  const pnpmJs = join(nodeDir, 'node_modules', 'pnpm', 'bin', 'pnpm.cjs')
  const cargoBin = join(process.env.USERPROFILE || '', '.cargo', 'bin', 'cargo.exe')
  const pathKey = Object.keys(process.env).find((key) => key.toLowerCase() === 'path') || 'Path'
  const spawnEnv = {
    ...process.env,
    [pathKey]: `${nodeDir};${dirname(cargoBin)};${process.env[pathKey] || ''}`
  }
  const coreEnv = { ...process.env, ...spawnEnv, CORE_PORT: String(corePort), CORE_DATABASE_URL: dbUrl, CORE_BACKUP_INTERVAL_HOURS: '0' }
  const appEnv = { ...process.env, ...spawnEnv, CORE_URL: `http://127.0.0.1:${corePort}`, PORT: String(appPort) }

  const core = spawn(cargoBin, ['run', '-q', '-p', 'logholizon-core'], { cwd: repoRoot, env: coreEnv, stdio: 'inherit', shell: false })
  core.on('error', (error) => console.error('core spawn error', error))
  // Spawn the Nuxt dev server through the pnpm-installed nuxt.CMD shim via
  // cmd.exe. The shim sets NODE_PATH for pnpm's isolated layout (tailwindcss
  // resolution depends on it). Pass the whole command as one verbatim string
  // so cmd.exe parses the quoted shim path correctly.
  const nuxtCmd = join(appDir, 'node_modules', '.bin', 'nuxt.CMD')
  const app = spawn('cmd.exe', [`/d /s /c ""${nuxtCmd}" dev --port ${appPort}"`], {
    cwd: appDir,
    env: appEnv,
    stdio: 'inherit',
    shell: false,
    windowsVerbatimArguments: true
  })
  app.on('error', (error) => console.error('app spawn error', error))

  let failed = false
  try {
    await run(cargoBin, ['run', '-q', '-p', 'logholizon-cli', '--', 'seed', '--demo'], {
      cwd: repoRoot,
      env: { ...process.env, ...spawnEnv, CORE_DATABASE_URL: dbUrl }
    })
    await waitFor(`http://127.0.0.1:${corePort}/health`, 120_000)
    await waitFor(`http://localhost:${appPort}/login`, 180_000)
    // Drop the runner's own `--` separator before forwarding args to the
    // Playwright CLI; otherwise every spec path is treated as a filter miss
    // and zero tests run (or the wrong file pattern is used).
    const forwarded = process.argv.slice(2).filter((arg) => arg !== '--')
    await run(process.execPath, [join(appDir, 'node_modules', '@playwright', 'test', 'cli.js'), 'test', ...forwarded], {
      cwd: appDir,
      env: { ...process.env, ...spawnEnv, PLAYWRIGHT_BASE_URL: `http://localhost:${appPort}` }
    })
  } catch (error) {
    console.error(error)
    failed = true
  } finally {
    core.kill()
    app.kill()
    process.exit(failed ? 1 : 0)
  }
}

main()
