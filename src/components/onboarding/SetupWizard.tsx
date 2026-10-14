import React, { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Check, Loader2, Mic, ClipboardList, Sparkles } from "lucide-react";
import OshabeTextLogo from "../icons/OshabeTextLogo";
import { Button } from "../ui/Button";
import { useModelStore } from "../../stores/modelStore";
import { useSettings } from "../../hooks/useSettings";
import { useOsType } from "../../hooks/useOsType";
import { formatKeyCombination } from "../../lib/utils/keyboard";
import ModelPicker from "./ModelPicker";
import { pickPresets, type LanguageChoice, type PresetId } from "./presets";

interface SetupWizardProps {
  onModelSelected: () => void;
  preview?: boolean;
}

type Step = "purpose" | "quality" | "practice" | "cleanup" | "advanced";
type Purpose = "dictation" | "meetings" | "both";

const PRESET_ORDER: PresetId[] = ["fast", "balanced", "best"];

const SetupWizard: React.FC<SetupWizardProps> = ({
  onModelSelected,
  preview = false,
}) => {
  const { t } = useTranslation();
  const osType = useOsType();
  const { settings, updateSetting } = useSettings();
  const {
    models,
    downloadModel,
    selectModel,
    downloadingModels,
    verifyingModels,
    extractingModels,
    downloadProgress,
  } = useModelStore();

  const [step, setStep] = useState<Step>("purpose");
  const [purpose, setPurpose] = useState<Purpose>("both");
  const [language, setLanguage] = useState<LanguageChoice>("english");
  const [choice, setChoice] = useState<string | null>(null);
  const [cleanup, setCleanup] = useState<boolean>(false);
  const [practice, setPractice] = useState("");
  const started = useRef(false);

  const presets = useMemo(
    () => pickPresets(models, language),
    [models, language],
  );

  const hotkey = formatKeyCombination(
    settings?.bindings?.transcribe?.current_binding ?? "",
    osType,
  );

  // Once the chosen model is on disk, select it and move to the practice box.
  useEffect(() => {
    if (preview || !choice || started.current) return;
    const model = models.find((m) => m.id === choice);
    const busy =
      choice in downloadingModels ||
      choice in verifyingModels ||
      choice in extractingModels;
    if (model?.is_downloaded && !busy) {
      started.current = true;
      selectModel(choice).then((ok) => {
        if (ok) {
          setStep("practice");
        } else {
          toast.error(t("onboarding.errors.selectModel"));
          started.current = false;
          setChoice(null);
        }
      });
    }
  }, [
    choice,
    models,
    downloadingModels,
    verifyingModels,
    extractingModels,
    selectModel,
    preview,
    t,
  ]);

  const pick = async (id: string) => {
    if (preview || choice) return;
    setChoice(id);
    const model = models.find((m) => m.id === id);
    if (model?.is_downloaded) return;
    const ok = await downloadModel(id);
    if (!ok) setChoice(null);
  };

  const finish = async () => {
    if (!preview) {
      // Auto-detect guesses wrong for accented English (it can pick Hindi), so
      // lock to English when that is what the person said they speak.
      if (language === "english")
        await updateSetting("selected_language", "en");
      if (cleanup) {
        await updateSetting("post_process_enabled", true);
        await updateSetting("always_cleanup_enabled", true);
      }
    }
    onModelSelected();
  };

  const shell = (children: React.ReactNode) => (
    <div className="h-screen w-full flex flex-col p-6 gap-4 overflow-y-auto">
      <div className="flex flex-col items-center gap-2 shrink-0">
        <OshabeTextLogo width={200} />
      </div>
      <div className="max-w-[560px] w-full mx-auto flex-1 flex flex-col gap-4">
        {children}
      </div>
    </div>
  );

  const option = (
    selected: boolean,
    onClick: () => void,
    title: string,
    body: string,
    icon?: React.ReactNode,
  ) => (
    <button
      type="button"
      onClick={onClick}
      className={`w-full text-left rounded-xl border p-4 flex gap-3 items-start transition-colors cursor-pointer ${
        selected
          ? "border-logo-primary bg-logo-primary/10"
          : "border-mid-gray/30 hover:border-logo-primary/60"
      }`}
    >
      {icon}
      <span className="flex-1">
        <span className="block font-semibold">{title}</span>
        <span className="block text-sm text-text/70">{body}</span>
      </span>
      {selected && <Check className="w-5 h-5 text-logo-primary shrink-0" />}
    </button>
  );

  if (step === "advanced") {
    return (
      <ModelPicker
        onModelSelected={() => setStep("practice")}
        preview={preview}
      />
    );
  }

  if (step === "purpose") {
    return shell(
      <>
        <h1 className="text-xl font-semibold text-center">
          {t("onboarding.wizard.purposeTitle")}
        </h1>
        <div className="flex flex-col gap-3">
          {option(
            purpose === "dictation",
            () => setPurpose("dictation"),
            t("onboarding.wizard.purposeDictation"),
            t("onboarding.wizard.purposeDictationBody"),
            <Mic className="w-5 h-5 mt-1 shrink-0" />,
          )}
          {option(
            purpose === "meetings",
            () => setPurpose("meetings"),
            t("onboarding.wizard.purposeMeetings"),
            t("onboarding.wizard.purposeMeetingsBody"),
            <ClipboardList className="w-5 h-5 mt-1 shrink-0" />,
          )}
          {option(
            purpose === "both",
            () => setPurpose("both"),
            t("onboarding.wizard.purposeBoth"),
            t("onboarding.wizard.purposeBothBody"),
            <Sparkles className="w-5 h-5 mt-1 shrink-0" />,
          )}
        </div>
        <Button className="mt-2" onClick={() => setStep("quality")}>
          {t("onboarding.wizard.next")}
        </Button>
      </>,
    );
  }

  if (step === "quality") {
    const downloading = choice !== null;
    const pct = choice ? downloadProgress[choice]?.percentage : undefined;
    return shell(
      <>
        <h1 className="text-xl font-semibold text-center">
          {t("onboarding.wizard.qualityTitle")}
        </h1>
        <p className="text-center text-sm text-text/70">
          {t("onboarding.wizard.qualityBody")}
        </p>
        <div className="flex gap-2 justify-center">
          {(["english", "multi"] as LanguageChoice[]).map((l) => (
            <button
              key={l}
              type="button"
              disabled={downloading}
              onClick={() => setLanguage(l)}
              className={`px-3 py-1 rounded-full border text-sm cursor-pointer disabled:opacity-50 ${
                language === l
                  ? "border-logo-primary bg-logo-primary/10"
                  : "border-mid-gray/30"
              }`}
            >
              {t(`onboarding.wizard.lang.${l}`)}
            </button>
          ))}
        </div>
        <div className="flex flex-col gap-3">
          {PRESET_ORDER.map((id) => {
            const model = presets[id];
            if (!model) return null;
            const active = choice === model.id;
            return (
              <div key={id}>
                {option(
                  active,
                  () => void pick(model.id),
                  t(`onboarding.wizard.preset.${id}`),
                  t(`onboarding.wizard.preset.${id}Body`, {
                    size:
                      model.size_mb >= 1000
                        ? `${(model.size_mb / 1024).toFixed(1)} GB`
                        : `${Math.round(model.size_mb)} MB`,
                  }),
                )}
                {active && (
                  <div className="mt-2 flex items-center gap-2 text-sm text-text/70">
                    <Loader2 className="w-4 h-4 animate-spin" />
                    <div className="flex-1 h-2 rounded-full bg-mid-gray/20 overflow-hidden">
                      <div
                        className="h-full bg-logo-primary transition-all"
                        style={{ width: `${Math.max(2, pct ?? 2)}%` }}
                      />
                    </div>
                    <span>
                      {pct !== undefined ? `${Math.round(pct)}%` : ""}
                    </span>
                  </div>
                )}
              </div>
            );
          })}
        </div>
        <p className="text-xs text-text/60 text-center">
          {t("onboarding.wizard.offlineNote")}
        </p>
        <button
          type="button"
          disabled={downloading}
          onClick={() => setStep("advanced")}
          className="text-sm text-text/60 hover:text-text underline mx-auto cursor-pointer disabled:opacity-50"
        >
          {t("onboarding.wizard.chooseMyself")}
        </button>
      </>,
    );
  }

  if (step === "practice") {
    return shell(
      <>
        <h1 className="text-xl font-semibold text-center">
          {t("onboarding.wizard.practiceTitle")}
        </h1>
        <p className="text-center text-sm text-text/70">
          {t("onboarding.wizard.practiceBody", { hotkey })}
        </p>
        <textarea
          autoFocus
          value={practice}
          onChange={(e) => setPractice(e.target.value)}
          placeholder={t("onboarding.wizard.practicePlaceholder")}
          className="w-full min-h-[140px] rounded-xl border border-mid-gray/30 bg-transparent p-3 focus:outline-none focus:border-logo-primary"
        />
        <Button onClick={() => setStep("cleanup")}>
          {practice.trim()
            ? t("onboarding.wizard.next")
            : t("onboarding.wizard.skip")}
        </Button>
      </>,
    );
  }

  return shell(
    <>
      <h1 className="text-xl font-semibold text-center">
        {t("onboarding.wizard.cleanupTitle")}
      </h1>
      <p className="text-center text-sm text-text/70">
        {t("onboarding.wizard.cleanupBody")}
      </p>
      <div className="flex flex-col gap-3">
        {option(
          !cleanup,
          () => setCleanup(false),
          t("onboarding.wizard.cleanupOff"),
          t("onboarding.wizard.cleanupOffBody"),
        )}
        {option(
          cleanup,
          () => setCleanup(true),
          t("onboarding.wizard.cleanupOn"),
          t("onboarding.wizard.cleanupOnBody"),
        )}
      </div>
      {purpose !== "dictation" && (
        <p className="text-xs text-text/60 text-center">
          {t("onboarding.wizard.meetingsHint")}
        </p>
      )}
      <Button onClick={() => void finish()}>
        {t("onboarding.wizard.done")}
      </Button>
    </>,
  );
};

export default SetupWizard;
