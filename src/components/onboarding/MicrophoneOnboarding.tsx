import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import OshabeTextLogo from "../icons/OshabeTextLogo";
import MicTest from "../shared/MicTest";
import { MicrophoneSelector } from "../settings/MicrophoneSelector";
import { Button } from "../ui/Button";
import { useSettings } from "../../hooks/useSettings";

interface MicrophoneOnboardingProps {
  onComplete: () => void;
  preview?: boolean;
}

const MicrophoneOnboarding: React.FC<MicrophoneOnboardingProps> = ({
  onComplete,
  preview = false,
}) => {
  const { t } = useTranslation();
  const { refreshAudioDevices } = useSettings();
  const [testing, setTesting] = useState(false);
  const [heard, setHeard] = useState(false);

  useEffect(() => {
    if (!preview) void refreshAudioDevices();
  }, [preview, refreshAudioDevices]);

  return (
    <div className="h-screen w-full flex flex-col items-center p-6 gap-5">
      <OshabeTextLogo width={200} />
      <div className="max-w-[480px] w-full flex flex-col gap-4 text-center">
        <div>
          <h1 className="text-lg font-semibold">
            {t("onboarding.microphone.title")}
          </h1>
          <p className="text-text/70 text-sm mt-1">
            {t("onboarding.microphone.subtitle")}
          </p>
        </div>
        <div
          className={`text-start rounded-lg border border-mid-gray/20 ${
            testing ? "opacity-50 pointer-events-none" : ""
          }`}
        >
          <MicrophoneSelector descriptionMode="tooltip" grouped={true} />
        </div>
        <MicTest
          preview={preview}
          onRunningChange={setTesting}
          onHeard={() => setHeard(true)}
        />
        <div className="flex justify-center gap-2">
          <Button
            variant={heard ? "primary" : "secondary"}
            size="lg"
            onClick={onComplete}
          >
            {heard
              ? t("onboarding.microphone.continue")
              : t("onboarding.microphone.skip")}
          </Button>
        </div>
      </div>
    </div>
  );
};

export default MicrophoneOnboarding;
