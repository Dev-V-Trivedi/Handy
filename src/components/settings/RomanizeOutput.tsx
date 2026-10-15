import React from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../../hooks/useSettings";
import { ToggleSwitch } from "../ui/ToggleSwitch";

export const RomanizeOutput: React.FC = React.memo(() => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();

  return (
    <ToggleSwitch
      checked={getSetting("romanize_output") ?? false}
      onChange={(value) => updateSetting("romanize_output", value)}
      isUpdating={isUpdating("romanize_output")}
      label={t("settings.modelSettings.romanize.label")}
      description={t("settings.modelSettings.romanize.description")}
      descriptionMode="tooltip"
      grouped={true}
    />
  );
});

RomanizeOutput.displayName = "RomanizeOutput";
