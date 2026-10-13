import assert from "node:assert/strict";
import type { ModelInfo } from "@/bindings";
import { pickPresets } from "./presets";

const m = (o: Partial<ModelInfo>): ModelInfo =>
  ({
    id: "x",
    is_custom: false,
    supports_streaming: false,
    supports_language_selection: true,
    accuracy_score: 0.5,
    speed_score: 0.5,
    size_mb: 100,
    ...o,
  }) as ModelInfo;

const models = [
  m({ id: "tiny", speed_score: 0.95, accuracy_score: 0.4, size_mb: 50 }),
  m({ id: "mid", speed_score: 0.7, accuracy_score: 0.75 }),
  m({ id: "big", speed_score: 0.3, accuracy_score: 0.95, size_mb: 3000 }),
  m({
    id: "stream",
    speed_score: 1,
    accuracy_score: 1,
    supports_streaming: true,
  }),
];

const p = pickPresets(models, "english");
assert.equal(p.fast?.id, "tiny");
assert.equal(p.balanced?.id, "mid");
assert.equal(p.best?.id, "big");

const single = pickPresets([m({ id: "only" })], "english");
assert.deepEqual(
  Object.values(single).map((x) => x?.id),
  ["only"],
);

assert.deepEqual(pickPresets([], "multi"), {});
console.log("presets tests passed");
