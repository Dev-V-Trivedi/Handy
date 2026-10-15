import assert from "node:assert/strict";
import type { ModelInfo } from "@/bindings";
import { pickPresets } from "./presets";

const m = (o: Partial<ModelInfo>): ModelInfo =>
  ({
    id: "x",
    is_custom: false,
    supports_streaming: false,
    supports_language_selection: true,
    supported_languages: ["en", "hi"],
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

const p = pickPresets(models, []);
assert.equal(p.fast?.id, "tiny");
assert.equal(p.balanced?.id, "mid");
assert.equal(p.best?.id, "big");

const single = pickPresets([m({ id: "only" })], []);
assert.deepEqual(
  Object.values(single).map((x) => x?.id),
  ["only"],
);

assert.deepEqual(pickPresets([], ["hi"]), {});
// A language the models do not list leaves nothing to recommend.
assert.deepEqual(pickPresets(models, ["te"]), {});
// Hindi keeps models that list it, and drops Whisper Small.
const hindi = pickPresets(
  [m({ id: "small" }), m({ id: "medium", accuracy_score: 0.7 })],
  ["hi"],
);
assert.deepEqual(
  Object.values(hindi).map((x) => x?.id),
  ["medium"],
);
console.log("presets tests passed");
