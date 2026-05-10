import { describe, it, expect, beforeAll } from "bun:test";
import { getProbesContext, checkCommand, parseGateResponseRaw, recordTest } from "./probes-context";
import type { ProbesContext } from "./probes-context";

let ctx: ProbesContext;

beforeAll(async () => {
  ctx = await getProbesContext();
});

describe("allow_list + safe commands", () => {
  it("allows safe echo command", async () => {
    await recordTest(ctx.p, "allow_list + safe commands > allows safe echo command", async () => {
      const res = await ctx.p.unix.send({
        data: checkCommand({ command: "echo", args: ["hello world"], cwd: "/tmp", pid: 9999 }),
        timeout_ms: 10000,
      });
      const response = parseGateResponseRaw(res);
      expect(response.action).toBe("allow");
    });
  });

  it("allows git status", async () => {
    await recordTest(ctx.p, "allow_list + safe commands > allows git status", async () => {
      const res = await ctx.p.unix.send({
        data: checkCommand({ command: "git", args: ["status"], cwd: "/tmp", pid: 9999 }),
        timeout_ms: 10000,
      });
      const response = parseGateResponseRaw(res);
      expect(response.action).toBe("allow");
    });
  });

  it("allows ls -la", async () => {
    await recordTest(ctx.p, "allow_list + safe commands > allows ls -la", async () => {
      const res = await ctx.p.unix.send({
        data: checkCommand({ command: "ls", args: ["-la"], cwd: "/tmp", pid: 9999 }),
        timeout_ms: 10000,
      });
      const response = parseGateResponseRaw(res);
      expect(response.action).toBe("allow");
    });
  });
});
