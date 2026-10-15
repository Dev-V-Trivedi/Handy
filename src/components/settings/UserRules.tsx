import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../../hooks/useSettings";
import type { UserRule } from "@/bindings";
import { Button } from "../ui/Button";
import { Textarea } from "../ui/Textarea";
import { ToggleSwitch } from "../ui/ToggleSwitch";

const newRuleId = () => `urule_${Date.now().toString(36)}`;
const SCOPES = ["all", "meetings", "dictation"] as const;

export const UserRules: React.FC = React.memo(() => {
  const { t } = useTranslation();
  const { getSetting, updateSetting } = useSettings();

  const stored = getSetting("user_rules");
  const [rules, setRules] = useState<UserRule[]>(stored ?? []);

  // Keep local edits in sync when the stored rules change from elsewhere.
  useEffect(() => {
    setRules(stored ?? []);
  }, [stored]);

  const commit = (next: UserRule[]) => {
    setRules(next);
    updateSetting("user_rules", next);
  };

  const editLocal = (id: string, patch: Partial<UserRule>) =>
    setRules((prev) => prev.map((r) => (r.id === id ? { ...r, ...patch } : r)));

  const patchAndSave = (id: string, patch: Partial<UserRule>) =>
    commit(rules.map((r) => (r.id === id ? { ...r, ...patch } : r)));

  return (
    <div className="px-4 py-3 space-y-3">
      <p className="text-xs text-mid-gray">
        {t("settings.postProcessing.userRules.description")}
      </p>
      {rules.map((rule) => (
        <div
          key={rule.id}
          className="space-y-2 rounded-lg border border-mid-gray/20 p-3"
        >
          <Textarea
            variant="compact"
            className="w-full"
            value={rule.text}
            placeholder={t("settings.postProcessing.userRules.placeholder")}
            aria-label={t("settings.postProcessing.userRules.ruleLabel")}
            onChange={(e) => editLocal(rule.id, { text: e.target.value })}
            onBlur={() => commit(rules)}
          />
          <div className="flex flex-wrap items-center gap-2">
            <select
              className="rounded-md border border-mid-gray/30 bg-transparent px-2 py-1 text-sm"
              value={rule.scope}
              aria-label={t("settings.postProcessing.userRules.scopeLabel")}
              onChange={(e) => patchAndSave(rule.id, { scope: e.target.value })}
            >
              {SCOPES.map((scope) => (
                <option key={scope} value={scope}>
                  {t(`settings.postProcessing.userRules.scope.${scope}`)}
                </option>
              ))}
            </select>
            <ToggleSwitch
              checked={rule.enabled}
              onChange={(value) => patchAndSave(rule.id, { enabled: value })}
              isUpdating={false}
              label={t("settings.postProcessing.userRules.enabled")}
              description=""
              descriptionMode="tooltip"
              grouped={true}
            />
            <Button
              variant="danger-ghost"
              size="sm"
              onClick={() => commit(rules.filter((r) => r.id !== rule.id))}
            >
              {t("settings.postProcessing.userRules.remove")}
            </Button>
          </div>
        </div>
      ))}
      <Button
        variant="secondary"
        size="sm"
        onClick={() =>
          commit([
            ...rules,
            { id: newRuleId(), text: "", scope: "all", enabled: true },
          ])
        }
      >
        {t("settings.postProcessing.userRules.add")}
      </Button>
    </div>
  );
});
UserRules.displayName = "UserRules";
