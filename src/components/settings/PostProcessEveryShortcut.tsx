import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface PostProcessEveryShortcutProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

/** Run the selected prompt after every transcribe shortcut. */
export const PostProcessEveryShortcut: React.FC<PostProcessEveryShortcutProps> =
  React.memo(({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();
    const enabled = getSetting("post_process_every_shortcut") ?? false;

    return (
      <ToggleSwitch
        checked={enabled}
        onChange={(next) => updateSetting("post_process_every_shortcut", next)}
        isUpdating={isUpdating("post_process_every_shortcut")}
        label={t("settings.postProcessing.everyShortcut.title")}
        description={t("settings.postProcessing.everyShortcut.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  });
