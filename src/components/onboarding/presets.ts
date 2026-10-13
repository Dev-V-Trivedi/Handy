import type { ModelInfo } from "@/bindings";

export type PresetId = "fast" | "balanced" | "best";
export type LanguageChoice = "english" | "multi";

/**
 * Pick one model per quality preset so people never have to read model names.
 *
 * Streaming-only models are skipped: meeting notes transcribe in batches and
 * those models are not reliable there. Legacy and custom models are skipped too.
 * The choice is made from the catalog scores, so new models are picked up
 * without code changes.
 */
export function pickPresets(
  models: ModelInfo[],
  language: LanguageChoice,
): Partial<Record<PresetId, ModelInfo>> {
  const pool = models.filter(
    (m) =>
      !m.is_custom &&
      !m.supports_streaming &&
      (language === "english" ? true : m.supports_language_selection) &&
      m.accuracy_score > 0 &&
      m.speed_score > 0,
  );
  if (pool.length === 0) return {};

  const best = [...pool].sort(
    (a, b) =>
      b.accuracy_score - a.accuracy_score || b.speed_score - a.speed_score,
  )[0];
  const fast = [...pool].sort(
    (a, b) =>
      b.speed_score - a.speed_score || b.accuracy_score - a.accuracy_score,
  )[0];
  const balanced = [...pool].sort(
    (a, b) =>
      b.accuracy_score * b.speed_score - a.accuracy_score * a.speed_score ||
      a.size_mb - b.size_mb,
  )[0];

  const out: Partial<Record<PresetId, ModelInfo>> = { fast, balanced, best };
  // Drop duplicates so two cards never point at the same model.
  if (out.fast?.id === out.balanced?.id) delete out.fast;
  if (out.best?.id === out.balanced?.id) delete out.best;
  return out;
}
