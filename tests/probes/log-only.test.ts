import { describe, it } from "bun:test";

describe("log_only", () => {
  it.todo("rejects safe commands when log_only=true — returns action=reject");
  it.todo("audit trail populates stage_chain and matched_list columns");
  it.todo("LLM metadata logged — model_name, confidence, deliberation_raw in audit");
  it.todo("catch_list match logged — matched_list=catch_list, matched_pattern set");
  it.todo("human approval fires — approval_id returned in response");
});
