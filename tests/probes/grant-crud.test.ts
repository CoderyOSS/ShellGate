import { describe, it } from "bun:test";

describe("grant_crud", () => {
  it.todo("creates grant and allows matching command via allow_list");
  it.todo("expired grant does not match — command falls through to block");
  it.todo("revoked grant does not match — command falls through to block");
  it.todo("max_uses exhausted after limit reached");
});
