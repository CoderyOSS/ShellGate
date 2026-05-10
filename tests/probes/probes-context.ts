import { probes } from "@codery/probes";
import { spawn, type Subprocess } from "bun";
import { unlinkSync, writeFileSync, existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const SOCKET = "/tmp/gate-probes.sock";
const DB_PATH = "/tmp/gate-probes.db";
const CONFIG_PATH = "/tmp/gate-probes-config.toml";
const PROOF_PATH = join(import.meta.dir, "proof-records.md");

let _ctx: ProbesContext | null = null;
let _initPromise: Promise<ProbesContext> | null = null;
let _cleanedUp = false;

export interface ProbesContext {
  p: Awaited<ReturnType<typeof probes>>;
  configPath: string;
  server: Subprocess;
}

export interface GateResponse {
  action: string;
  env?: Record<string, string>;
  approval_id?: string;
  reason?: string;
}

export async function getProbesContext(): Promise<ProbesContext> {
  if (_ctx) return _ctx;
  if (!_initPromise) _initPromise = initProbes();
  _ctx = await _initPromise;
  return _ctx;
}

async function initProbes(): Promise<ProbesContext> {
  const config = `
[gate]
socket_path = "${SOCKET}"
db_path = "${DB_PATH}"
audit_ttl_secs = 7776000
rest_port = 0
rest_host = "127.0.0.1"
pending_queue_max = 100
allowed_uids = []

[github]
app_id = 0
app_key_path = ""
installation_id = 0

[telegram]
bot_token = ""
chat_id = 0

[mcp]
fifo_path = "/tmp/gate-mcp-probes.fifo"

[web]
dist_path = "/tmp/gate-web-probes"

[pipeline.llm]
model_name = "deepseek-chat"
api_url = "http://127.0.0.1:19876/v1/chat/completions"
api_key = "test"
max_tokens = 256
temperature = 0.1

[pipeline.stages.allow_list]
sampling_rate = 0.0

[pipeline.stages.catch_list]
patterns = ["rm -rf *", "auth:*", "curl * | *"]

[pipeline.stages.llm]
confidence_allow = 0.7
confidence_block = 0.3
max_context_commands = 50
warning_signs = []

[pipeline.stages.human]
timeout_seconds = 30

[pipeline.flows]
command_check = ["catch_list", "allow_list", "llm", "human"]
mcp_request = ["allow_list", "human"]
interactive_bootstrap = []
`;
  writeFileSync(CONFIG_PATH, config);

  const server = spawn({
    cmd: ["target/debug/gate-server"],
    env: { GATE_CONFIG: CONFIG_PATH },
    stdout: "pipe",
    stderr: "pipe",
    cwd: join(import.meta.dir, "..", ".."),
  });

  await new Promise((r) => setTimeout(r, 800));

  const p = await probes({
    unix: { client: { path: SOCKET, timeout_ms: 30000 } },
    sql: { path: DB_PATH },
    http: { server: { port: 19876 } },
    record: { output_path: PROOF_PATH },
  });

  const now = new Date().toISOString();
  const future = new Date(Date.now() + 86400000).toISOString();

  await p.sql.put({
    table: "agendas",
    rows: [
      {
        id: "probes-test-agenda",
        source: "probes-test",
        description: "probes e2e test agenda",
        scope: null,
        status: "active",
        created_at: now,
        expires_at: future,
      },
    ],
  });

  await p.sql.put({
    table: "derived_grants",
    rows: [
      {
        id: "probes-grant-echo",
        agenda_id: "probes-test-agenda",
        command_pattern: "echo",
        args_pattern: "*",
        path_pattern: null,
        notification: "silent",
        reason: "probes test grant for echo",
        confidence: 0.95,
        created_at: now,
        expires_at: future,
      },
      {
        id: "probes-grant-git",
        agenda_id: "probes-test-agenda",
        command_pattern: "git",
        args_pattern: "status",
        path_pattern: null,
        notification: "silent",
        reason: "probes test grant for git",
        confidence: 0.95,
        created_at: now,
        expires_at: future,
      },
      {
        id: "probes-grant-ls",
        agenda_id: "probes-test-agenda",
        command_pattern: "ls",
        args_pattern: "*",
        path_pattern: null,
        notification: "silent",
        reason: "probes test grant for ls",
        confidence: 0.95,
        created_at: now,
        expires_at: future,
      },
    ],
  });

  const ctx: ProbesContext = { p, configPath: CONFIG_PATH, server };

  process.on("beforeExit", async () => {
    if (_cleanedUp) return;
    _cleanedUp = true;
    try {
      await ctx.p.record.write();
      await ctx.p.close();
    } catch {}
    ctx.server.kill();
    for (const path of [SOCKET, DB_PATH, CONFIG_PATH]) {
      try {
        unlinkSync(path);
      } catch {}
    }
    if (existsSync(PROOF_PATH)) {
      console.log(`Proof records written to ${PROOF_PATH}`);
      console.log(`Size: ${readFileSync(PROOF_PATH).length} bytes`);
    }
  });

  return ctx;
}

export function checkCommand(req: {
  command: string;
  args: string[];
  cwd: string;
  pid: number;
}): string {
  return JSON.stringify({
    type: "check_command",
    request: req,
  });
}

export function parseGateResponseRaw(raw: string): GateResponse {
  const buf = Buffer.from(raw);
  if (buf.length < 4) {
    throw new Error(`response too short: ${buf.length} bytes`);
  }
  const len = buf.readUInt32BE(0);
  const body = buf.subarray(4, 4 + len);
  return JSON.parse(body.toString());
}

export async function recordTest(
  p: ProbesContext["p"],
  testName: string,
  fn: () => Promise<void>,
) {
  p.record.begin({ test_name: testName });
  try {
    await fn();
    p.record.end({ result: "pass" });
  } catch (e) {
    p.record.end({ result: "fail", error: String(e) });
    throw e;
  }
}
