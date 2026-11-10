import React, { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Trash2 } from "lucide-react";
import { commands, type ModelInfo, type ShortcutBinding } from "@/bindings";
import { formatKeyCombination } from "@/lib/utils/keyboard";
import {
  LANGUAGES,
  languageName,
  languagesOf,
} from "@/lib/constants/languages";
import { useSettings } from "@/hooks/useSettings";
import { useOsType } from "@/hooks/useOsType";
import { useShortcutCapture } from "@/hooks/useShortcutCapture";
import { usePressAway } from "@/hooks/usePressAway";
import { useModelStore } from "@/stores/modelStore";
import { Dropdown } from "../ui/Dropdown";
import { Button } from "../ui/Button";
import { SettingContainer } from "../ui/SettingContainer";

const CUSTOM_PREFIX = "transcribe_lang_";

const isCustom = (id: string) =>
  id.startsWith(CUSTOM_PREFIX) && id.length > CUSTOM_PREFIX.length;

/** Numeric order, so transcribe_lang_10 sorts after transcribe_lang_2. */
const byNumber = (a: ShortcutBinding, b: ShortcutBinding) =>
  Number(a.id.slice(CUSTOM_PREFIX.length)) -
  Number(b.id.slice(CUSTOM_PREFIX.length));

interface RowProps {
  binding: ShortcutBinding;
  models: ModelInfo[];
  captureOnMount: boolean;
  onChanged: () => Promise<void>;
}

const LanguageShortcutRow: React.FC<RowProps> = ({
  binding,
  models,
  captureOnMount,
  onChanged,
}) => {
  const { t, i18n } = useTranslation();
  const osType = useOsType();
  const { capturing, partial, start, cancel } = useShortcutCapture(
    binding.id,
    osType,
  );
  const keyRef = useRef<HTMLDivElement | null>(null);
  usePressAway(capturing, keyRef, cancel);

  useEffect(() => {
    if (captureOnMount) start();
    // Only when the row first appears.
  }, []);

  const language = binding.language ?? "";
  const model = binding.model ?? "";
  const modelInfo = models.find((m) => m.id === model);

  // A model that names its languages offers only those.
  const languageCodes = useMemo(
    () =>
      modelInfo && modelInfo.supported_languages.length > 0
        ? languagesOf(modelInfo.supported_languages).map((l) => l.code)
        : LANGUAGES.map((l) => l.code),
    [modelInfo],
  );

  const languageOptions = [
    { value: "", label: t("settings.general.languageShortcuts.sameLanguage") },
    ...languageCodes.map((code) => ({
      value: code,
      label: languageName(code, i18n.language),
    })),
  ];
  if (language && !languageCodes.includes(language)) {
    languageOptions.push({
      value: language,
      label: t("settings.general.languageShortcuts.unsupported", {
        language: languageName(language, i18n.language),
      }),
    });
  }

  const modelOptions = [
    { value: "", label: t("settings.general.languageShortcuts.sameModel") },
    ...models
      .filter((m) => m.is_downloaded || m.id === model)
      .map((m) => ({
        value: m.id,
        label: m.is_downloaded
          ? m.name
          : t("settings.general.languageShortcuts.notDownloaded", {
              model: m.name,
            }),
      })),
  ];

  const saveProfile = async (nextLanguage: string, nextModel: string) => {
    const result = await commands.changeTranscribeShortcutProfile(
      binding.id,
      nextLanguage || null,
      nextModel || null,
    );
    if (result.status === "error") console.error(result.error);
    await onChanged();
  };

  const remove = async () => {
    if (capturing) cancel();
    const result = await commands.removeTranscribeShortcut(binding.id);
    if (result.status === "error") console.error(result.error);
    await onChanged();
  };

  return (
    <div className="flex flex-wrap items-center gap-2 py-1">
      {capturing ? (
        <div
          ref={keyRef}
          className="px-2 py-1 text-sm font-semibold border border-logo-primary bg-logo-primary/20 rounded-md min-w-24"
        >
          {partial
            ? formatKeyCombination(partial, osType)
            : t("settings.general.shortcut.pressKeys")}
        </div>
      ) : (
        <div
          className="px-2 py-1 text-sm font-semibold bg-wash border border-line-strong hover:bg-logo-primary/10 rounded-md cursor-pointer hover:border-logo-primary min-w-24"
          onClick={start}
          title={t("settings.general.languageShortcuts.recordKey")}
        >
          {binding.current_binding
            ? formatKeyCombination(binding.current_binding, osType)
            : t("settings.general.languageShortcuts.noKey")}
        </div>
      )}
      <Dropdown
        className="min-w-36"
        options={languageOptions}
        selectedValue={language}
        onSelect={(value) => saveProfile(value, model)}
      />
      <Dropdown
        className="min-w-44"
        options={modelOptions}
        selectedValue={model}
        onSelect={(value) => {
          // Keep the language only if the new model can hear it.
          const next = models.find((m) => m.id === value);
          const supported =
            !next ||
            next.supported_languages.length === 0 ||
            languagesOf(next.supported_languages).some(
              (l) => l.code === language,
            );
          saveProfile(supported ? language : "", value);
        }}
      />
      <button
        type="button"
        className="p-1.5 rounded-md text-ink-3 hover:text-text hover:bg-wash cursor-pointer"
        aria-label={t("settings.general.languageShortcuts.remove")}
        title={t("settings.general.languageShortcuts.remove")}
        onClick={remove}
      >
        <Trash2 className="size-[14px]" aria-hidden="true" />
      </button>
    </div>
  );
};

/**
 * Extra transcribe shortcuts, each dictating in its own language and with
 * its own model (e.g. one key for English on Parakeet, another for Russian
 * on Whisper). The language and model apply to that recording only.
 */
export const LanguageShortcuts: React.FC<{ grouped?: boolean }> = ({
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();
  const models = useModelStore((s) => s.models);
  const [justAdded, setJustAdded] = useState<string | null>(null);

  const rows = Object.values(getSetting("bindings") ?? {})
    .filter((b): b is ShortcutBinding => !!b && isCustom(b.id))
    .sort(byNumber);

  const add = async () => {
    const result = await commands.addTranscribeShortcut(null, null);
    if (result.status === "error") {
      console.error(result.error);
      return;
    }
    await refreshSettings();
    setJustAdded(result.data.id);
  };

  return (
    <SettingContainer
      title={t("settings.general.languageShortcuts.title")}
      description={t("settings.general.languageShortcuts.description")}
      descriptionMode="tooltip"
      grouped={grouped}
      layout="stacked"
    >
      <div className="flex flex-col gap-1">
        {rows.map((binding) => (
          <LanguageShortcutRow
            key={binding.id}
            binding={binding}
            models={models}
            captureOnMount={binding.id === justAdded}
            onChanged={refreshSettings}
          />
        ))}
        <div>
          <Button variant="secondary" size="sm" onClick={add}>
            <span className="inline-flex items-center gap-1">
              <Plus className="size-[14px]" aria-hidden="true" />
              {t("settings.general.languageShortcuts.add")}
            </span>
          </Button>
        </div>
      </div>
    </SettingContainer>
  );
};
