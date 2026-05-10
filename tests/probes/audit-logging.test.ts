import { describe, it } from "bun:test";

describe("audit_logging", () => {
  it.todo("allowed command writes audit row with granted_by and exit_code");
  it.todo("blocked command writes audit row with granted_by=catch_list");
  it.todo("LLM decision writes audit row with model_name, confidence, deliberation_raw");
});
