import React, { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { Check, Download, Loader2, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { commands, type LocalLlmStatus } from "@/bindings";
import { useSettings } from "../../hooks/useSettings";
import Badge from "../ui/Badge";
import { Button } from "../ui/Button";

interface Progress {
  step: "runner" | "model";
  percent: number;
}

/**
 * Built-in local cleanup models. One button downloads the small model runner
 * (once) and the model, then turns cleanup on. No Ollama, no account, no key.
 */
export const BuiltInModels: React.FC = () => {
  const { t } = useTranslation();
  const { settings, updateSetting } = useSettings();
  const [status, setStatus] = useState<LocalLlmStatus | null>(null);
  const [progress, setProgress] = useState<Record<string, Progress>>({});
  const working = useRef<Set<string>>(new Set());

  const refresh = useCallback(async () => {
    const res = await commands.localLlmStatus();
    if (res.status === "ok") setStatus(res.data);
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    const unlisten = listen<{ id: string; completed: number; total: number }>(
      "local-llm-progress",
      (event) => {
        const { id, completed, total } = event.payload;
        const percent = total > 0 ? Math.round((completed / total) * 100) : 0;
        setProgress((prev) => {
          const next = { ...prev };
          for (const key of Object.keys(next)) {
            if (id === "runner") next[key] = { step: "runner", percent };
            else if (key === id) next[key] = { step: "model", percent };
          }
          return next;
        });
      },
    );
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  const activate = async (id: string) => {
    const res = await commands.localLlmUseModel(id);
    if (res.status === "error") {
      toast.error(t("settings.postProcessing.builtIn.failed"), {
        description: res.error,
      });
      return;
    }
    if (!settings?.post_process_selected_prompt_id) {
      await updateSetting("post_process_selected_prompt_id", "flow_cleanup");
    }
    if (!settings?.post_process_enabled) {
      await updateSetting("post_process_enabled", true);
    }
    toast.success(t("settings.postProcessing.local.ready"));
    void refresh();
  };

  const install = async (id: string) => {
    if (working.current.has(id)) return;
    working.current.add(id);
    setProgress((prev) => ({ ...prev, [id]: { step: "runner", percent: 0 } }));
    try {
      if (!status?.runner_installed) {
        const r = await commands.localLlmInstallRunner();
        if (r.status === "error") throw new Error(r.error);
      }
      setProgress((prev) => ({ ...prev, [id]: { step: "model", percent: 0 } }));
      const m = await commands.localLlmDownloadModel(id);
      if (m.status === "error") throw new Error(m.error);
      await activate(id);
    } catch (e) {
      toast.error(t("settings.postProcessing.builtIn.failed"), {
        description: e instanceof Error ? e.message : String(e),
      });
    } finally {
      working.current.delete(id);
      setProgress((prev) => {
        const next = { ...prev };
        delete next[id];
        return next;
      });
      void refresh();
    }
  };

  const remove = async (id: string) => {
    await commands.localLlmDeleteModel(id);
    void refresh();
  };

  if (!status) return null;

  const activeId =
    settings?.post_process_provider_id === "oshabe_local"
      ? (settings?.post_process_models?.["oshabe_local"] ?? "")
      : "";

  return (
    <div className="space-y-3 px-4 py-3">
      <p className="text-sm text-text/70">
        {t("settings.postProcessing.builtIn.intro")}
      </p>
      {status.models.map((model) => {
        const p = progress[model.id];
        const busy = p !== undefined;
        const active = model.installed && activeId === model.id;
        return (
          <div
            key={model.id}
            className="flex flex-col gap-2 rounded-lg border-2 border-mid-gray/20 p-4"
          >
            <div className="flex items-start justify-between gap-3">
              <div className="flex flex-wrap items-center gap-2">
                <h3 className="text-base font-semibold text-text">
                  {model.name}
                </h3>
                {active && (
                  <Badge variant="success">
                    <Check className="mr-1 h-3 w-3" />
                    {t("settings.postProcessing.local.inUse")}
                  </Badge>
                )}
              </div>
              <span className="shrink-0 text-xs text-text/50">
                {t("settings.postProcessing.local.size", {
                  size: (model.size_mb / 1024).toFixed(1),
                })}
              </span>
            </div>
            {busy && (
              <div className="w-full">
                <div className="h-1.5 w-full overflow-hidden rounded-full bg-mid-gray/20">
                  <div
                    className="h-full rounded-full bg-logo-primary transition-all duration-300"
                    style={{ width: `${p.percent}%` }}
                  />
                </div>
                <div className="mt-1 text-xs text-text/60">
                  {t(
                    p.step === "runner"
                      ? "settings.postProcessing.builtIn.stepRunner"
                      : "settings.postProcessing.builtIn.stepModel",
                    { percent: p.percent },
                  )}
                </div>
              </div>
            )}
            <div className="flex items-center gap-2">
              {!model.installed && !busy && (
                <Button
                  size="sm"
                  variant="primary"
                  onClick={() => void install(model.id)}
                >
                  <span className="flex items-center gap-1.5">
                    <Download className="h-3.5 w-3.5" />
                    {t("settings.postProcessing.local.install")}
                  </span>
                </Button>
              )}
              {busy && (
                <span className="flex items-center gap-1.5 text-sm text-text/60">
                  <Loader2 className="h-3.5 w-3.5 animate-spin" />
                </span>
              )}
              {model.installed && !active && (
                <Button
                  size="sm"
                  variant="primary"
                  onClick={() => void activate(model.id)}
                >
                  {t("settings.postProcessing.local.use")}
                </Button>
              )}
              {model.installed && (
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => void remove(model.id)}
                  className="ms-auto text-logo-primary/85 hover:text-logo-primary hover:bg-logo-primary/10"
                >
                  <span className="flex items-center gap-1.5">
                    <Trash2 className="h-3.5 w-3.5" />
                    {t("common.delete")}
                  </span>
                </Button>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
};
