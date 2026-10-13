import React from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../../hooks/useSettings";
import { ToggleSwitch } from "../ui/ToggleSwitch";

export const AlwaysCleanup: React.FC = React.memo(() => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();

  return (
    <ToggleSwitch
      checked={getSetting("always_cleanup_enabled") ?? false}
      onChange={(value) => updateSetting("always_cleanup_enabled", value)}
      isUpdating={isUpdating("always_cleanup_enabled")}
      label={t("settings.postProcessing.always.label")}
      description={t("settings.postProcessing.always.description")}
      descriptionMode="tooltip"
      grouped={true}
    />
  );
});

AlwaysCleanup.displayName = "AlwaysCleanup";
