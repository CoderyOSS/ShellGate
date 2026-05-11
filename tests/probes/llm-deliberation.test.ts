import { describe, it, expect, beforeAll, afterAll } from "bun:test";
import { setupTestFile, checkCommand, parseGateResponseRaw, recordTest } from "./probes-context";
import type { ProbesContext } from "./probes-context";

let ctx: ProbesContext;

beforeAll(async () => {
  ctx = await setupTestFile();
});

afterAll(async () => {
  await ctx.teardown();
});

describe("llm deliberation", () => {
  it("captures LLM prompt and allows when LLM says ALLOW", async () => {
    await recordTest(ctx.p, "llm deliberation > captures prompt and allows", async () => {
      await ctx.p.http.put({
        status: 200,
        body: {
          choices: [
            {
              message: {
                content:
                  "DECISION: ALLOW\nCONFIDENCE: 0.90\nREASON: command fits the active agenda",
              },
            },
          ],
        },
      });

      const res = await ctx.p.unix.send({
        data: checkCommand({ command: "unknown-tool", args: ["--help"], cwd: "/tmp", pid: 9999 }),
        timeout_ms: 10000,
      });
      const response = parseGateResponseRaw(res);

      expect(response.action).toBe("allow");

      const requests = await ctx.p.http.read();
      expect(requests.length).toBeGreaterThanOrEqual(1);
      expect(requests[0].method).toBe("POST");
      const promptBody = requests[0].body;
      expect(promptBody).toBeTruthy();
      expect(promptBody!).toContain("security gatekeeper");
      expect(promptBody!).toContain("unknown-tool");
    });
  });

  it("blocks when LLM returns BLOCK verdict", async () => {
    await recordTest(ctx.p, "llm deliberation > blocks on LLM BLOCK verdict", async () => {
      await ctx.p.http.put({
        status: 200,
        body: {
          choices: [
            {
              message: {
                content:
                  "DECISION: BLOCK\nCONFIDENCE: 0.95\nREASON: dangerous operation detected",
              },
            },
          ],
        },
      });

      const res = await ctx.p.unix.send({
        data: checkCommand({ command: "unknown-dangerous", args: [], cwd: "/tmp", pid: 9999 }),
        timeout_ms: 10000,
      });
      const response = parseGateResponseRaw(res);

      expect(response.action).toBe("reject");
      expect(response.reason).toContain("dangerous");

      const requests = await ctx.p.http.read();
      expect(requests.length).toBeGreaterThanOrEqual(1);
    });
  });
});
