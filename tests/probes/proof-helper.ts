import { p } from "@codery/probes";
import { it as bunIt } from "bun:test";

export function it(name: string, fn: (...args: any[]) => void | Promise<void>) {
  bunIt(name, async () => {
    p.proof.begin(name);
    try {
      await fn();
    } finally {
      p.proof.end();
    }
  });
}
