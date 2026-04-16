import { chromium, type Browser, type Page } from 'playwright';
import { readFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse as parseYaml } from 'yaml';

export type StepStatus = 'pass' | 'fail';

export interface StepResult {
  name: string;
  status: StepStatus;
  duration_ms: number;
  error?: string;
}

export interface FlowResult {
  flow: string;
  status: StepStatus;
  duration_ms: number;
  steps: StepResult[];
  artifacts?: Record<string, string>;
}

export interface VerifyContext {
  browser: Browser;
  page: Page;
  baseUrl: string;
  results: StepResult[];
  step<T>(name: string, fn: () => Promise<T>): Promise<T>;
}

export type FlowFn = (ctx: VerifyContext) => Promise<void>;

interface ManifestFlow {
  name: string;
  description?: string;
  timeout_s?: number;
  tags?: string[];
}

interface Manifest {
  version: number;
  runner_command: string;
  base_url?: string;
  flows: ManifestFlow[];
}

const here = dirname(fileURLToPath(import.meta.url));
const verifyRoot = join(here, '..');
const flowsDir = join(here, 'flows');

function loadManifest(): Manifest {
  const text = readFileSync(join(verifyRoot, 'workflows.yaml'), 'utf8');
  return parseYaml(text) as Manifest;
}

function flowImplemented(name: string): boolean {
  return existsSync(join(flowsDir, `${name}.ts`)) ||
         existsSync(join(flowsDir, `${name}.js`));
}

async function loadFlow(name: string): Promise<FlowFn> {
  const mod = await import(`./flows/${name}.js`);
  if (typeof mod.default !== 'function') {
    throw new Error(`flow '${name}' has no default-exported function`);
  }
  return mod.default as FlowFn;
}

function listFlows(): void {
  const manifest = loadManifest();
  const flows = manifest.flows.map((f) => ({
    name: f.name,
    description: f.description ?? '',
    timeout_s: f.timeout_s ?? 60,
    tags: f.tags ?? [],
    implemented: flowImplemented(f.name),
  }));
  process.stdout.write(JSON.stringify({ flows }) + '\n');
  process.exit(0);
}

async function runFlow(flowName: string): Promise<void> {
  if (!flowImplemented(flowName)) {
    const result: FlowResult = {
      flow: flowName,
      status: 'fail',
      duration_ms: 0,
      steps: [{
        name: 'load-flow',
        status: 'fail',
        duration_ms: 0,
        error: `flow '${flowName}' has no implementation in src/flows/`,
      }],
    };
    process.stdout.write(JSON.stringify(result) + '\n');
    process.exit(1);
  }

  const baseUrl = process.env.VERIFY_BASE_URL ??
                  loadManifest().base_url ??
                  'http://localhost:3000';
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  const results: StepResult[] = [];

  const ctx: VerifyContext = {
    browser,
    page,
    baseUrl,
    results,
    async step(name, fn) {
      const t0 = Date.now();
      try {
        const out = await fn();
        results.push({ name, status: 'pass', duration_ms: Date.now() - t0 });
        return out;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        results.push({ name, status: 'fail', duration_ms: Date.now() - t0, error: msg });
        throw err;
      }
    },
  };

  const flowStart = Date.now();
  let status: StepStatus = 'pass';
  let urlAtFailure: string | undefined;

  try {
    const flow = await loadFlow(flowName);
    await flow(ctx);
  } catch {
    status = 'fail';
    urlAtFailure = ctx.page.url();
  } finally {
    await browser.close();
  }

  const result: FlowResult = {
    flow: flowName,
    status,
    duration_ms: Date.now() - flowStart,
    steps: results,
    ...(urlAtFailure ? { artifacts: { url_at_failure: urlAtFailure } } : {}),
  };

  process.stdout.write(JSON.stringify(result) + '\n');
  process.exit(status === 'pass' ? 0 : 1);
}

async function main() {
  const arg = process.argv[2];
  if (!arg) {
    process.stderr.write('usage: runner <flow-name> | --list\n');
    process.exit(2);
  }

  if (arg === '--list') {
    listFlows();
    return;
  }

  await runFlow(arg);
}

main().catch((err) => {
  process.stderr.write(`runner crashed: ${err instanceof Error ? err.message : String(err)}\n`);
  process.exit(3);
});
