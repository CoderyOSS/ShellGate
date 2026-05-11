import type { UnixActions } from "@codery/probes";

export interface GateRequest {
  command: string;
  args: string[];
  cwd: string;
  pid: number;
}

export interface GateResponse {
  action: string;
  env?: Record<string, string>;
  approval_id?: string;
  reason?: string;
}

export const gateAdapter = (raw: UnixActions) => ({
  async send(params: { data: GateRequest; path?: string; timeout_ms?: number }): Promise<GateResponse> {
    const wire = JSON.stringify({ type: "check_command", request: params.data });
    const rawResp = await raw.send({ data: wire, path: params.path, timeout_ms: params.timeout_ms });
    const buf = Buffer.from(rawResp);
    if (buf.length < 4) throw new Error(`response too short: ${buf.length} bytes`);
    const len = buf.readUInt32BE(0);
    return JSON.parse(buf.subarray(4, 4 + len).toString());
  },
});
