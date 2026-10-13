import type { MeetingDetail } from "@/bindings";
import {
  formatClock,
  type FormatLabels,
  type SummaryData,
} from "./meetingFormat";

/** Format-neutral document: both Word and PDF are built from this. */
export interface ExportBlock {
  heading?: string;
  paragraphs?: string[];
  bullets?: string[];
  /** Transcript lines: speaker and time are shown in bold before the text. */
  lines?: { who: string; time: string; text: string }[];
}

export interface ExportDoc {
  title: string;
  subtitle: string;
  blocks: ExportBlock[];
}

export const buildExportDoc = (
  detail: MeetingDetail,
  summary: SummaryData | null,
  labels: FormatLabels,
): ExportDoc => {
  const blocks: ExportBlock[] = [];
  if (summary?.summary)
    blocks.push({ heading: labels.summary, paragraphs: [summary.summary] });
  if (summary?.keyPoints?.length)
    blocks.push({ heading: labels.keyPoints, bullets: summary.keyPoints });
  if (summary?.decisions?.length)
    blocks.push({ heading: labels.decisions, bullets: summary.decisions });
  if (summary?.actionItems?.length)
    blocks.push({
      heading: labels.actionItems,
      bullets: summary.actionItems.map((a) => {
        const extra = [a.owner, a.due].filter(Boolean).join(", ");
        return extra ? `${a.task} (${extra})` : a.task;
      }),
    });
  if (summary?.openQuestions?.length)
    blocks.push({
      heading: labels.openQuestions,
      bullets: summary.openQuestions,
    });
  blocks.push({
    heading: labels.transcript,
    lines: detail.segments.map((s) => ({
      who: s.track === "you" ? labels.you : labels.others,
      time: formatClock(s.start_ms),
      text: s.text,
    })),
  });
  return {
    title: detail.title,
    subtitle: new Date(detail.started_at * 1000).toLocaleString(),
    blocks,
  };
};

export const buildDocx = async (doc: ExportDoc): Promise<Uint8Array> => {
  const { Document, Packer, Paragraph, TextRun, HeadingLevel } = await import(
    "docx"
  );
  const children: InstanceType<typeof Paragraph>[] = [
    new Paragraph({ text: doc.title, heading: HeadingLevel.TITLE }),
    new Paragraph({
      children: [new TextRun({ text: doc.subtitle, italics: true })],
    }),
  ];
  for (const b of doc.blocks) {
    if (b.heading)
      children.push(
        new Paragraph({ text: b.heading, heading: HeadingLevel.HEADING_1 }),
      );
    for (const p of b.paragraphs ?? [])
      children.push(new Paragraph({ text: p, spacing: { after: 120 } }));
    for (const i of b.bullets ?? [])
      children.push(new Paragraph({ text: i, bullet: { level: 0 } }));
    for (const l of b.lines ?? [])
      children.push(
        new Paragraph({
          spacing: { after: 80 },
          children: [
            new TextRun({ text: `${l.who} ${l.time}  `, bold: true }),
            new TextRun({ text: l.text }),
          ],
        }),
      );
  }
  const file = new Document({ sections: [{ children }] });
  return new Uint8Array(await Packer.toArrayBuffer(file));
};

interface PdfMakeLike {
  addVirtualFileSystem: (vfs: unknown) => void;
  createPdf: (def: unknown) => { getBuffer: () => Promise<Uint8Array> };
}

/**
 * PDF uses the Roboto font that ships with pdfmake: Latin, Cyrillic and Greek.
 * Other scripts (Hindi, Chinese, Arabic...) should use the Word export.
 */
export const buildPdf = async (doc: ExportDoc): Promise<Uint8Array> => {
  const pdfMake = (await import("pdfmake/build/pdfmake"))
    .default as unknown as PdfMakeLike;
  const vfs = (await import("pdfmake/build/vfs_fonts")).default as unknown;
  pdfMake.addVirtualFileSystem(vfs);

  const content: unknown[] = [
    { text: doc.title, style: "title" },
    { text: doc.subtitle, italics: true, margin: [0, 0, 0, 12] },
  ];
  for (const b of doc.blocks) {
    if (b.heading) content.push({ text: b.heading, style: "h" });
    for (const p of b.paragraphs ?? [])
      content.push({ text: p, margin: [0, 0, 0, 6] });
    if (b.bullets?.length)
      content.push({ ul: b.bullets, margin: [0, 0, 0, 6] });
    for (const l of b.lines ?? [])
      content.push({
        text: [{ text: `${l.who} ${l.time}  `, bold: true }, l.text],
        margin: [0, 0, 0, 4],
      });
  }
  return pdfMake
    .createPdf({
      content,
      defaultStyle: { fontSize: 10.5 },
      styles: {
        title: { fontSize: 20, bold: true, margin: [0, 0, 0, 4] },
        h: { fontSize: 14, bold: true, margin: [0, 12, 0, 6] },
      },
    })
    .getBuffer();
};
