import type { MeetingDetail } from "@/bindings";

export interface SummaryData {
  title?: string;
  summary?: string;
  keyPoints?: string[];
  decisions?: string[];
  actionItems?: { task: string; owner?: string | null; due?: string | null }[];
  openQuestions?: string[];
}

export interface FormatLabels {
  you: string;
  others: string;
  summary: string;
  keyPoints: string;
  decisions: string;
  actionItems: string;
  openQuestions: string;
  transcript: string;
}

/** "mm:ss", or "h:mm:ss" past an hour. */
export const formatClock = (ms: number): string => {
  const total = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const pad = (n: number) => n.toString().padStart(2, "0");
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${pad(m)}:${pad(s)}`;
};

export const parseSummary = (raw: string | null): SummaryData | null => {
  if (!raw) return null;
  try {
    const value = JSON.parse(raw) as unknown;
    return value && typeof value === "object" ? (value as SummaryData) : null;
  } catch {
    return null;
  }
};

export const buildTranscript = (
  detail: MeetingDetail,
  labels: FormatLabels,
): string =>
  detail.segments
    .map(
      (s) =>
        `[${formatClock(s.start_ms)}] ${
          s.track === "you" ? labels.you : labels.others
        }: ${s.text}`,
    )
    .join("\n");

const bullets = (items?: string[]): string =>
  items && items.length ? items.map((i) => `- ${i}`).join("\n") + "\n\n" : "";

/** Markdown notes: title, summary sections, then the transcript. */
export const buildMarkdown = (
  detail: MeetingDetail,
  summary: SummaryData | null,
  labels: FormatLabels,
): string => {
  let out = `# ${detail.title}\n\n`;
  out += `${new Date(detail.started_at * 1000).toLocaleString()}\n\n`;
  if (summary) {
    if (summary.summary)
      out += `## ${labels.summary}\n\n${summary.summary}\n\n`;
    if (summary.keyPoints?.length)
      out += `## ${labels.keyPoints}\n\n${bullets(summary.keyPoints)}`;
    if (summary.decisions?.length)
      out += `## ${labels.decisions}\n\n${bullets(summary.decisions)}`;
    if (summary.actionItems?.length) {
      out += `## ${labels.actionItems}\n\n`;
      out +=
        summary.actionItems
          .map((a) => {
            const extra = [a.owner, a.due].filter(Boolean).join(", ");
            return `- [ ] ${a.task}${extra ? ` (${extra})` : ""}`;
          })
          .join("\n") + "\n\n";
    }
    if (summary.openQuestions?.length)
      out += `## ${labels.openQuestions}\n\n${bullets(summary.openQuestions)}`;
  }
  out += `## ${labels.transcript}\n\n${buildTranscript(detail, labels)}\n`;
  return out;
};
