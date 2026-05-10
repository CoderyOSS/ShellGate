import { describe, it } from "bun:test";

describe("permissions", () => {
  it.todo("OFF default blocks command despite no catch_list match");
  it.todo("ON default allows command without grant");
  it.todo("NEEDS_APPROVAL default reaches human stage with approval_id");
  it.todo("active grant overrides OFF default for same action");
});
