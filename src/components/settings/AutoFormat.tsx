import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface AutoFormatProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

/** Final period, capital first letter, and the model's own punctuation. */
export const AutoFormat: React.FC<AutoFormatProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();
    const enabled = getSetting("auto_format") ?? true;

    return (
      <ToggleSwitch
        checked={enabled}
        onChange={(next) => updateSetting("auto_format", next)}
        isUpdating={isUpdating("auto_format")}
        label={t("settings.advanced.autoFormat.title")}
        description={t("settings.advanced.autoFormat.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  },
);
