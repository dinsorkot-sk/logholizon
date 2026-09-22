const { spawn } = require('node:child_process')
const { existsSync, mkdirSync } = require('node:fs')
const { homedir, tmpdir } = require('node:os')
const { delimiter, dirname, join } = require('node:path')

const scriptDir = __dirname
// tests/e2e -> app (two levels up), then packages (three), then repo root (four).
const appDir = join(scriptDir, '..', '..')
const repoRoot = join(scriptDir, '..', '..', '..', '..')

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
  const isWindows = process.platform === 'win32'
  const nodeDir = dirname(process.execPath)
  const home = process.env.USERPROFILE || homedir() || ''
  const windowsCargo = home ? join(home, '.cargo', 'bin', 'cargo.exe') : ''
  const cargoBin = isWindows && windowsCargo && existsSync(windowsCargo) ? windowsCargo : 'cargo'
  const pathKey = Object.keys(process.env).find((key) => key.toLowerCase() === 'path') || 'Path'
  const pathSep = isWindows ? ';' : delimiter
  const spawnEnv = {
    ...process.env,
    [pathKey]: `${nodeDir}${pathSep}${process.env[pathKey] || ''}`
  }
  const coreEnv = { ...process.env, ...spawnEnv, CORE_PORT: String(corePort), CORE_DATABASE_URL: dbUrl, CORE_BACKUP_INTERVAL_HOURS: '0' }
  const coreUrl = `http://127.0.0.1:${corePort}`
  const appEnv = { ...process.env, ...spawnEnv, CORE_URL: coreUrl, NUXT_CORE_URL: coreUrl, PORT: String(appPort) }

  const coreBinary = !isWindows && existsSync(join(repoRoot, 'target', 'debug', 'logholizon-core'))
    ? join(repoRoot, 'target', 'debug', 'logholizon-core')
    : cargoBin
  const coreArgs = coreBinary === cargoBin ? ['run', '-q', '-p', 'logholizon-core'] : []
  const core = spawn(coreBinary, coreArgs, { cwd: repoRoot, env: coreEnv, stdio: 'inherit', shell: false })
  core.on('error', (error) => console.error('core spawn error', error))
  core.on('exit', (code, signal) => console.error(`core exited before E2E: code=${code} signal=${signal}`))
  // Windows: spawn the Nuxt dev server through the pnpm-installed nuxt.CMD
  // shim via cmd.exe. The shim sets NODE_PATH for pnpm's isolated layout
  // (tailwindcss resolution depends on it). Pass the whole command as one
  // verbatim string so cmd.exe parses the quoted shim path correctly.
  // POSIX (Linux/macOS CI): spawn the nuxt binary directly.
  const nuxtBin = isWindows
    ? join(appDir, 'node_modules', '.bin', 'nuxt.CMD')
    : join(appDir, 'node_modules', '.bin', 'nuxt')
  const appMode = process.env.E2E_APP_MODE === 'preview' ? 'preview' : 'dev'
  const appArgs = appMode === 'preview'
    ? [join(appDir, '.output', 'server', 'index.mjs')]
    : ['dev', '--host', '127.0.0.1', '--port', String(appPort)]
  const appCommand = appMode === 'preview' ? process.execPath : nuxtBin
  const app = isWindows && appMode !== 'preview'
    ? spawn('cmd.exe', [`/d /s /c \"\"${nuxtBin}\" ${appArgs.join(' ')}\"`], {
      cwd: appDir,
      env: appEnv,
      stdio: 'inherit',
      shell: false,
      windowsVerbatimArguments: true
    })
    : spawn(appCommand, appArgs, {
      cwd: appDir,
      env: appEnv,
      stdio: 'inherit',
      shell: false
    })
  app.on('error', (error) => console.error('app spawn error', error))

  let failed = false
  try {
    // The core process runs migrations on startup. Wait for health before
    // seeding so the CLI cannot race the migration runner on the same SQLite DB.
    await waitFor(`http://127.0.0.1:${corePort}/health`, 120_000)
    const cliBinary = !isWindows && existsSync(join(repoRoot, 'target', 'debug', 'logholizon-cli'))
      ? join(repoRoot, 'target', 'debug', 'logholizon-cli')
      : cargoBin
    const cliArgs = cliBinary === cargoBin ? ['run', '-q', '-p', 'logholizon-cli', '--', 'seed', '--demo'] : ['seed', '--demo']
    await run(cliBinary, cliArgs, {
      cwd: repoRoot,
      env: { ...process.env, ...spawnEnv, CORE_DATABASE_URL: dbUrl }
    })
    await waitFor(`http://127.0.0.1:${appPort}/login`, 180_000)
    await waitFor(`http://127.0.0.1:${corePort}/health`, 10_000)
    // Drop the runner's own `--` separator before forwarding args to the
    // Playwright CLI; otherwise every spec path is treated as a filter miss
    // and zero tests run (or the wrong file pattern is used).
    const forwarded = process.argv.slice(2).filter((arg) => arg !== '--')
    await run(process.execPath, [join(appDir, 'node_modules', '@playwright', 'test', 'cli.js'), 'test', ...forwarded], {
      cwd: appDir,
      env: { ...process.env, ...spawnEnv, PLAYWRIGHT_BASE_URL: `http://127.0.0.1:${appPort}` }
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
