import { p } from "@codery/probes";
import { describe, expect, beforeAll, afterAll } from "bun:test";
import { it } from "./proof-helper";
import { gateAdapter } from "./gate-adapter";

const gate = p.unix.use(gateAdapter);

beforeAll(async () => {
  await p.sql.put({ file: "./shared.fixture.yaml" });
});

afterAll(async () => {
  await p.sql.clear();
  p.proof.save();
});

describe("allow_list + safe commands", () => {
  it("allows safe echo command", async () => {
    const res = await gate.send({
      data: { command: "echo", args: ["hello world"], cwd: "/tmp", pid: 9999 },
    });
    expect(res.action).toBe("allow");
  });

  it("allows git status", async () => {
    const res = await gate.send({
      data: { command: "git", args: ["status"], cwd: "/tmp", pid: 9999 },
    });
    expect(res.action).toBe("allow");
  });

  it("allows ls -la", async () => {
    const res = await gate.send({
      data: { command: "ls", args: ["-la"], cwd: "/tmp", pid: 9999 },
    });
    expect(res.action).toBe("allow");
  });
});
