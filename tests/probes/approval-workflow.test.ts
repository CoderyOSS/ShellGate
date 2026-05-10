import { describe, it } from "bun:test";

describe("approval_workflow", () => {
  it.todo("unknown command reaches human stage — returns approval_id");
  it.todo("human approves via API — resolution propagates to pending channel");
  it.todo("human rejects via API — rejection propagates to pending channel");
  it.todo("double resolution is idempotent — second approve returns conflict");
  it.todo("approval expires — status transitions to expired after timeout");
});
