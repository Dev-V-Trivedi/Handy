import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { SummaryTemplate } from "@/bindings";
import { useSettings } from "../../../hooks/useSettings";
import { Button } from "../../ui/Button";
import { Input } from "../../ui/Input";
import { Textarea } from "../../ui/Textarea";

const BUILTIN_IDS = [
  "standup",
  "client_call",
  "interview",
  "lecture",
  "sales_call",
] as const;
const DEFAULT_ID = "default";

const newTemplateId = () => `custom_${Date.now().toString(36)}`;

/** Pick or write the template that shapes the AI meeting summary. */
export const SummaryTemplates: React.FC = React.memo(() => {
  const { t } = useTranslation();
  const { getSetting, updateSetting } = useSettings();

  const selectedId = getSetting("summary_template_id") ?? DEFAULT_ID;
  const stored = getSetting("summary_templates");
  const [templates, setTemplates] = useState<SummaryTemplate[]>(stored ?? []);
  const [editing, setEditing] = useState(false);

  useEffect(() => {
    setTemplates(stored ?? []);
  }, [stored]);

  const commit = (next: SummaryTemplate[]) => {
    setTemplates(next);
    void updateSetting("summary_templates", next);
  };

  const editLocal = (id: string, patch: Partial<SummaryTemplate>) =>
    setTemplates((prev) =>
      prev.map((tpl) => (tpl.id === id ? { ...tpl, ...patch } : tpl)),
    );

  const remove = (id: string) => {
    commit(templates.filter((tpl) => tpl.id !== id));
    if (selectedId === id)
      void updateSetting("summary_template_id", DEFAULT_ID);
  };

  const add = () => {
    const id = newTemplateId();
    commit([...templates, { id, name: "", instructions: "" }]);
    setEditing(true);
  };

  return (
    <div className="space-y-3 px-1">
      <label className="flex items-center justify-between gap-3 text-sm">
        <span>{t("meetings.templates.label")}</span>
        <select
          value={selectedId}
          onChange={(e) =>
            void updateSetting("summary_template_id", e.target.value)
          }
          className="max-w-[60%] rounded-lg border border-mid-gray/40 bg-transparent px-2 py-1"
        >
          <option value={DEFAULT_ID}>{t("meetings.templates.default")}</option>
          {BUILTIN_IDS.map((id) => (
            <option key={id} value={id}>
              {t(`meetings.templates.builtin.${id}`)}
            </option>
          ))}
          {templates
            .filter((tpl) => tpl.name.trim())
            .map((tpl) => (
              <option key={tpl.id} value={tpl.id}>
                {tpl.name}
              </option>
            ))}
        </select>
      </label>
      <p className="text-xs text-text/60">{t("meetings.templates.hint")}</p>

      <Button
        variant="secondary"
        size="sm"
        onClick={() => setEditing((v) => !v)}
        aria-expanded={editing}
      >
        {editing
          ? t("meetings.templates.hideEditor")
          : t("meetings.templates.showEditor")}
      </Button>

      {editing && (
        <div className="space-y-3">
          {templates.map((tpl) => (
            <div
              key={tpl.id}
              className="space-y-2 rounded-lg border border-mid-gray/20 p-3"
            >
              <Input
                className="w-full"
                value={tpl.name}
                placeholder={t("meetings.templates.namePlaceholder")}
                aria-label={t("meetings.templates.nameLabel")}
                maxLength={60}
                onChange={(e) => editLocal(tpl.id, { name: e.target.value })}
                onBlur={() => commit(templates)}
              />
              <Textarea
                variant="compact"
                className="w-full"
                value={tpl.instructions}
                placeholder={t("meetings.templates.instructionsPlaceholder")}
                aria-label={t("meetings.templates.instructionsLabel")}
                maxLength={1200}
                onChange={(e) =>
                  editLocal(tpl.id, { instructions: e.target.value })
                }
                onBlur={() => commit(templates)}
              />
              <Button
                variant="danger-ghost"
                size="sm"
                onClick={() => remove(tpl.id)}
              >
                {t("meetings.templates.remove")}
              </Button>
            </div>
          ))}
          <Button variant="secondary" size="sm" onClick={add}>
            {t("meetings.templates.add")}
          </Button>
        </div>
      )}
    </div>
  );
});
SummaryTemplates.displayName = "SummaryTemplates";
