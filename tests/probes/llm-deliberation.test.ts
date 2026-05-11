import { p } from "@codery/probes";
import { describe, it, expect, beforeAll, afterAll } from "bun:test";
import { gateAdapter } from "./gate-adapter";

const gate = p.unix.use(gateAdapter);

beforeAll(async () => {
  await p.sql.put({ file: "./shared.fixture.yaml" });
});

afterAll(async () => {
  await p.sql.clear();
  p.proof.save();
});

describe("llm deliberation", () => {
  it("captures LLM prompt and allows when LLM says ALLOW", async () => {
    await p.http.put({
      status: 200,
      body: {
        choices: [{
          message: {
            content: "DECISION: ALLOW\nCONFIDENCE: 0.90\nREASON: command fits the active agenda",
          },
        }],
      },
    });

    const res = await gate.send({
      data: { command: "unknown-tool", args: ["--help"], cwd: "/tmp", pid: 9999 },
    });

    expect(res.action).toBe("allow");

    const requests = await p.http.read();
    expect(requests.length).toBeGreaterThanOrEqual(1);
    expect(requests[0].method).toBe("POST");
    const promptBody = requests[0].body;
    expect(promptBody).toBeTruthy();
    expect(promptBody!).toContain("security gatekeeper");
    expect(promptBody!).toContain("unknown-tool");
  });

  it("blocks when LLM returns BLOCK verdict", async () => {
    await p.http.put({
      status: 200,
      body: {
        choices: [{
          message: {
            content: "DECISION: BLOCK\nCONFIDENCE: 0.95\nREASON: dangerous operation detected",
          },
        }],
      },
    });

    const res = await gate.send({
      data: { command: "unknown-dangerous", args: [], cwd: "/tmp", pid: 9999 },
    });

    expect(res.action).toBe("reject");
    expect(res.reason).toContain("dangerous");

    const requests = await p.http.read();
    expect(requests.length).toBeGreaterThanOrEqual(1);
  });
});
