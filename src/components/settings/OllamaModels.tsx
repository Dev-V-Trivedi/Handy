import React, { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Check, Download, Loader2, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { commands } from "@/bindings";
import { useSettings } from "../../hooks/useSettings";
import { useSettingsStore } from "@/stores/settingsStore";
import Badge from "../ui/Badge";
import { Button } from "../ui/Button";
import { Alert } from "../ui/Alert";

interface CatalogEntry {
  /** Ollama model name. */
  id: string;
  /** Key under settings.postProcessing.local.models. */
  key: string;
  sizeGb: number;
  recommended?: boolean;
}

// Small instruction models that run on an ordinary laptop and are good enough
// for cleaning up dictated text.
const CATALOG: CatalogEntry[] = [
  { id: "qwen2.5:3b", key: "qwen3b", sizeGb: 1.9, recommended: true },
  { id: "qwen2.5:1.5b", key: "qwen15b", sizeGb: 1.0 },
  { id: "llama3.2:3b", key: "llama3b", sizeGb: 2.0 },
  { id: "gemma2:2b", key: "gemma2b", sizeGb: 1.6 },
];

interface Progress {
  completed: number;
  total: number;
  status: string;
}

const OLLAMA_URL = "https://ollama.com/download";

export const OllamaModels: React.FC = () => {
  const { t } = useTranslation();
  const { settings, updateSetting } = useSettings();
  const setPostProcessProvider = useSettingsStore(
    (s) => s.setPostProcessProvider,
  );
  const updatePostProcessModel = useSettingsStore(
    (s) => s.updatePostProcessModel,
  );

  const [running, setRunning] = useState<boolean | null>(null);
  const [installed, setInstalled] = useState<string[]>([]);
  const [progress, setProgress] = useState<Record<string, Progress>>({});
  const pulling = useRef<Set<string>>(new Set());

  const refresh = useCallback(async () => {
    try {
      const res = await commands.ollamaStatus();
      if (res.status === "ok") {
        setRunning(res.data.running);
        setInstalled(res.data.models);
      }
    } catch {
      setRunning(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
    // While Ollama is not running, keep checking so the list wakes up as soon
    // as the user starts it.
    const timer = setInterval(() => {
      if (!pulling.current.size) void refresh();
    }, 4000);
    return () => clearInterval(timer);
  }, [refresh]);

  useEffect(() => {
    const unlisten = listen<{
      model: string;
      status: string;
      completed: number;
      total: number;
    }>("ollama-pull-progress", (event) => {
      const { model, status, completed, total } = event.payload;
      setProgress((prev) => ({
        ...prev,
        [model]: { completed, total, status },
      }));
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  const isInstalled = (id: string) =>
    installed.some((name) => name === id || name === `${id}:latest`);

  const currentModel = settings?.post_process_models?.["ollama"] ?? "";
  const providerId = settings?.post_process_provider_id ?? "";

  const install = async (id: string) => {
    pulling.current.add(id);
    setProgress((prev) => ({
      ...prev,
      [id]: { completed: 0, total: 0, status: "starting" },
    }));
    try {
      const res = await commands.ollamaPullModel(id);
      if (res.status === "error") {
        toast.error(t("settings.postProcessing.local.installFailed"), {
          description: res.error,
        });
      }
    } finally {
      pulling.current.delete(id);
      setProgress((prev) => {
        const next = { ...prev };
        delete next[id];
        return next;
      });
      void refresh();
    }
  };

  const cancel = (id: string) => {
    void commands.ollamaCancelPull(id);
  };

  const remove = async (id: string) => {
    const res = await commands.ollamaDeleteModel(id);
    if (res.status === "error") {
      toast.error(t("settings.postProcessing.local.deleteFailed"), {
        description: res.error,
      });
    }
    void refresh();
  };

  // One click: pick Ollama, pick the model, pick the cleanup prompt, switch
  // post-processing on. Non-technical users never see a provider form.
  const useModel = async (id: string) => {
    await setPostProcessProvider("ollama");
    await updatePostProcessModel("ollama", id);
    if (!settings?.post_process_selected_prompt_id) {
      await updateSetting("post_process_selected_prompt_id", "flow_cleanup");
    }
    if (!settings?.post_process_enabled) {
      await updateSetting("post_process_enabled", true);
    }
    toast.success(t("settings.postProcessing.local.ready"));
  };

  return (
    <div className="space-y-3 px-4 py-3">
      <p className="text-sm text-text/70">
        {t("settings.postProcessing.local.intro")}
      </p>

      {running === false && (
        <Alert variant="info" contained>
          <div className="flex flex-col gap-2">
            <span>{t("settings.postProcessing.local.notRunning")}</span>
            <div className="flex gap-2">
              <Button
                size="sm"
                variant="primary"
                onClick={() => void openUrl(OLLAMA_URL)}
              >
                {t("settings.postProcessing.local.getOllama")}
              </Button>
              <Button
                size="sm"
                variant="secondary"
                onClick={() => void refresh()}
              >
                {t("settings.postProcessing.local.checkAgain")}
              </Button>
            </div>
          </div>
        </Alert>
      )}

      {CATALOG.map((entry) => {
        const have = isInstalled(entry.id);
        const p = progress[entry.id];
        const busy = p !== undefined;
        const percent =
          p && p.total > 0 ? Math.round((p.completed / p.total) * 100) : 0;
        const active =
          have && providerId === "ollama" && currentModel === entry.id;
        return (
          <div
            key={entry.id}
            className="flex flex-col gap-2 rounded-lg border-2 border-mid-gray/20 p-4"
          >
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <div className="flex flex-wrap items-center gap-2">
                  <h3 className="text-base font-semibold text-text">
                    {t(
                      `settings.postProcessing.local.models.${entry.key}.name`,
                    )}
                  </h3>
                  {entry.recommended && (
                    <Badge variant="primary">
                      {t("settings.postProcessing.local.recommended")}
                    </Badge>
                  )}
                  {active && (
                    <Badge variant="success">
                      <Check className="mr-1 h-3 w-3" />
                      {t("settings.postProcessing.local.inUse")}
                    </Badge>
                  )}
                </div>
                <p className="text-sm leading-relaxed text-text/60">
                  {t(
                    `settings.postProcessing.local.models.${entry.key}.description`,
                  )}
                </p>
              </div>
              <span className="shrink-0 text-xs text-text/50">
                {t("settings.postProcessing.local.size", {
                  size: entry.sizeGb.toFixed(1),
                })}
              </span>
            </div>

            {busy && (
              <div className="w-full">
                <div className="h-1.5 w-full overflow-hidden rounded-full bg-mid-gray/20">
                  <div
                    className="h-full rounded-full bg-logo-primary transition-all duration-300"
                    style={{ width: `${percent}%` }}
                  />
                </div>
                <div className="mt-1 text-xs text-text/60">
                  {t("settings.postProcessing.local.installing", {
                    percent,
                  })}
                </div>
              </div>
            )}

            <div className="flex items-center gap-2">
              {!have && !busy && (
                <Button
                  size="sm"
                  variant="primary"
                  disabled={running !== true}
                  onClick={() => void install(entry.id)}
                >
                  <span className="flex items-center gap-1.5">
                    <Download className="h-3.5 w-3.5" />
                    {t("settings.postProcessing.local.install")}
                  </span>
                </Button>
              )}
              {busy && (
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={() => cancel(entry.id)}
                >
                  <span className="flex items-center gap-1.5">
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                    {t("settings.postProcessing.local.cancel")}
                  </span>
                </Button>
              )}
              {have && !active && (
                <Button
                  size="sm"
                  variant="primary"
                  onClick={() => void useModel(entry.id)}
                >
                  {t("settings.postProcessing.local.use")}
                </Button>
              )}
              {have && (
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => void remove(entry.id)}
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
