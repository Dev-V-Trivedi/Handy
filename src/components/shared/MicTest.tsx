import React, { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { Mic, Square } from "lucide-react";
import { commands } from "@/bindings";
import { Button } from "../ui/Button";

const BARS = 16;
// Loudest spectrum bucket (0..1) that counts as "sound heard".
const HEARD_THRESHOLD = 0.15;
const SILENT_AFTER_MS = 4000;
const AUTO_STOP_MS = 60000;

type Phase = "idle" | "starting" | "listening" | "heard" | "silent";

interface MicTestProps {
  /** Reports whether a test is running, so a parent can lock device pickers. */
  onRunningChange?: (running: boolean) => void;
  /** Reports the first time sound was heard during this mount. */
  onHeard?: () => void;
  /** Debug previews are inert: they never open the microphone. */
  preview?: boolean;
}

/**
 * Live microphone check. Opens the mic through the backend, shows the level
 * as waveform bars and confirms when sound is heard. Nothing is recorded.
 */
const MicTest: React.FC<MicTestProps> = ({
  onRunningChange,
  onHeard,
  preview = false,
}) => {
  const { t } = useTranslation();
  const [phase, setPhase] = useState<Phase>("idle");
  const [levels, setLevels] = useState<number[]>(Array(BARS).fill(0));
  const [error, setError] = useState<string | null>(null);
  const smoothed = useRef<number[]>(Array(BARS).fill(0));
  const heardRef = useRef(false);
  const runningRef = useRef(false);
  const silentTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const autoStopTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearTimers = () => {
    if (silentTimer.current) clearTimeout(silentTimer.current);
    if (autoStopTimer.current) clearTimeout(autoStopTimer.current);
    silentTimer.current = null;
    autoStopTimer.current = null;
  };

  const stop = useCallback(async () => {
    if (!runningRef.current) return;
    runningRef.current = false;
    clearTimers();
    await commands.stopMicTest();
    smoothed.current = Array(BARS).fill(0);
    setLevels(Array(BARS).fill(0));
    setPhase("idle");
    onRunningChange?.(false);
  }, [onRunningChange]);

  const start = async () => {
    if (preview || runningRef.current) return;
    setError(null);
    heardRef.current = false;
    setPhase("starting");
    const result = await commands.startMicTest();
    if (result.status === "error") {
      setError(result.error);
      setPhase("idle");
      return;
    }
    runningRef.current = true;
    onRunningChange?.(true);
    setPhase("listening");
    silentTimer.current = setTimeout(() => {
      if (!heardRef.current) setPhase("silent");
    }, SILENT_AFTER_MS);
    autoStopTimer.current = setTimeout(() => void stop(), AUTO_STOP_MS);
  };

  // Level stream from the backend while a test runs.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    listen<number[]>("mic-test-level", (event) => {
      if (!runningRef.current) return;
      const next = event.payload;
      let peak = 0;
      smoothed.current = smoothed.current.map((prev, i) => {
        const target = Math.min(1, next[i] || 0);
        peak = Math.max(peak, target);
        return prev * 0.6 + target * 0.4;
      });
      setLevels(smoothed.current.slice());
      if (peak > HEARD_THRESHOLD && !heardRef.current) {
        heardRef.current = true;
        setPhase("heard");
        onHeard?.();
      } else if (peak > HEARD_THRESHOLD) {
        setPhase((p) => (p === "silent" ? "heard" : p));
      }
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [onHeard]);

  // Never leave the microphone open when the panel goes away.
  useEffect(() => {
    return () => {
      clearTimers();
      if (runningRef.current) {
        runningRef.current = false;
        void commands.stopMicTest();
        onRunningChange?.(false);
      }
    };
  }, [onRunningChange]);

  const running = phase !== "idle" && phase !== "starting";
  const statusText = error
    ? t("micTest.error", { error })
    : phase === "heard"
      ? t("micTest.heard")
      : phase === "silent"
        ? t("micTest.silent")
        : phase === "listening"
          ? t("micTest.listening")
          : t("micTest.idle");
  const statusColor = error
    ? "text-red-500"
    : phase === "heard"
      ? "text-emerald-600 dark:text-emerald-400"
      : phase === "silent"
        ? "text-amber-600 dark:text-amber-400"
        : "text-text/60";

  return (
    <div className="w-full rounded-lg border border-mid-gray/20 p-4 flex flex-col gap-3">
      <div
        className="h-16 flex items-center justify-center gap-1"
        role="img"
        aria-label={t("micTest.waveformLabel")}
      >
        {levels.map((v, i) => (
          <i
            key={i}
            className={`w-1.5 rounded-full transition-[height] duration-75 ${
              phase === "heard" ? "bg-emerald-500" : "bg-logo-primary"
            }`}
            style={{
              height: `${Math.max(4, Math.min(64, 4 + Math.pow(v, 0.7) * 60))}px`,
              opacity: running ? 1 : 0.35,
            }}
          />
        ))}
      </div>
      <p
        className={`text-sm font-medium text-center ${statusColor}`}
        role="status"
      >
        {statusText}
      </p>
      <div className="flex justify-center">
        {running ? (
          <Button variant="secondary" size="md" onClick={() => void stop()}>
            <span className="inline-flex items-center gap-2">
              <Square className="w-4 h-4" aria-hidden="true" />
              {t("micTest.stop")}
            </span>
          </Button>
        ) : (
          <Button
            variant="primary"
            size="md"
            onClick={() => void start()}
            disabled={phase === "starting"}
          >
            <span className="inline-flex items-center gap-2">
              <Mic className="w-4 h-4" aria-hidden="true" />
              {t("micTest.start")}
            </span>
          </Button>
        )}
      </div>
    </div>
  );
};

export default MicTest;
