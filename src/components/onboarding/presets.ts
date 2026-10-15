import type { ModelInfo } from "@/bindings";
import { supportsLanguageCode } from "@/lib/constants/languages";

export type PresetId = "fast" | "balanced" | "best";

/** Languages where Whisper Small is too weak to recommend. */
const NEEDS_BIGGER_MODEL = [
  "hi",
  "te",
  "ta",
  "mr",
  "bn",
  "gu",
  "kn",
  "ml",
  "pa",
  "ur",
];

/**
 * Pick one model per quality preset so people never have to read model names.
 *
 * `required` is the list of language codes the person speaks. An empty list
 * means English only or no preference, so any model qualifies. Otherwise a
 * model must list every code, and Whisper Small is dropped for Indian
 * languages because it makes too many mistakes there.
 *
 * Streaming-only models are skipped: meeting notes transcribe in batches and
 * those models are not reliable there. Legacy and custom models are skipped too.
 * The choice is made from the catalog scores, so new models are picked up
 * without code changes.
 */
export function pickPresets(
  models: ModelInfo[],
  required: string[],
): Partial<Record<PresetId, ModelInfo>> {
  const pool = models.filter(
    (m) =>
      !m.is_custom &&
      !m.supports_streaming &&
      (required.length === 0 ||
        (m.supports_language_selection &&
          required.every((code) =>
            supportsLanguageCode(m.supported_languages ?? [], code),
          ))) &&
      !(
        m.id === "small" && required.some((c) => NEEDS_BIGGER_MODEL.includes(c))
      ) &&
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
