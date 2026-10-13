import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../../hooks/useSettings";
import type { AppToneRule } from "@/bindings";
import { Button } from "../ui/Button";
import { Input } from "../ui/Input";
import { Textarea } from "../ui/Textarea";
import { ToggleSwitch } from "../ui/ToggleSwitch";

const newRuleId = () => `rule_${Date.now().toString(36)}`;

export const ContextTone: React.FC = React.memo(() => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();

  const enabled = getSetting("context_tone_enabled") ?? false;
  const storedRules = getSetting("app_tone_rules");
  const [rules, setRules] = useState<AppToneRule[]>(storedRules ?? []);

  // Keep local edits in sync when the stored rules change from elsewhere.
  useEffect(() => {
    setRules(storedRules ?? []);
  }, [storedRules]);

  const commit = (next: AppToneRule[]) => {
    setRules(next);
    updateSetting("app_tone_rules", next);
  };

  const editLocal = (id: string, patch: Partial<AppToneRule>) =>
    setRules((prev) => prev.map((r) => (r.id === id ? { ...r, ...patch } : r)));

  return (
    <>
      <ToggleSwitch
        checked={enabled}
        onChange={(value) => updateSetting("context_tone_enabled", value)}
        isUpdating={isUpdating("context_tone_enabled")}
        label={t("settings.postProcessing.contextTone.label")}
        description={t("settings.postProcessing.contextTone.description")}
        descriptionMode="tooltip"
        grouped={true}
      />

      {enabled && (
        <div className="px-4 py-3 space-y-3">
          {rules.map((rule) => (
            <div
              key={rule.id}
              className="space-y-2 rounded-lg border border-mid-gray/20 p-3"
            >
              <div className="flex items-center gap-2">
                <Input
                  variant="compact"
                  className="flex-1"
                  value={rule.pattern}
                  placeholder={t(
                    "settings.postProcessing.contextTone.patternPlaceholder",
                  )}
                  aria-label={t(
                    "settings.postProcessing.contextTone.patternLabel",
                  )}
                  onChange={(e) =>
                    editLocal(rule.id, { pattern: e.target.value })
                  }
                  onBlur={() => commit(rules)}
                />
                <Button
                  variant="danger-ghost"
                  size="sm"
                  onClick={() => commit(rules.filter((r) => r.id !== rule.id))}
                >
                  {t("settings.postProcessing.contextTone.remove")}
                </Button>
              </div>
              <Textarea
                variant="compact"
                className="w-full"
                value={rule.tone}
                placeholder={t(
                  "settings.postProcessing.contextTone.tonePlaceholder",
                )}
                aria-label={t("settings.postProcessing.contextTone.toneLabel")}
                onChange={(e) => editLocal(rule.id, { tone: e.target.value })}
                onBlur={() => commit(rules)}
              />
            </div>
          ))}
          <Button
            variant="secondary"
            size="sm"
            onClick={() =>
              commit([...rules, { id: newRuleId(), pattern: "", tone: "" }])
            }
          >
            {t("settings.postProcessing.contextTone.add")}
          </Button>
          <p className="text-xs text-mid-gray">
            {t("settings.postProcessing.contextTone.hint")}
          </p>
        </div>
      )}
    </>
  );
});
ContextTone.displayName = "ContextTone";
