//! A recording's own language and model, from the shortcut that started it.
//!
//! Applied to a copy of the settings for that one recording, never written
//! back: the saved `selected_language` / `selected_model` stay what the user
//! picked in Settings.

use crate::settings::AppSettings;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordingOverride {
    pub language: Option<String>,
    pub model: Option<String>,
}

impl RecordingOverride {
    /// The override `binding_id` carries, if it sets a language or a model.
    pub fn for_binding(settings: &AppSettings, binding_id: &str) -> Option<Self> {
        let binding = settings.bindings.get(binding_id)?;
        let pick = |v: &Option<String>| v.clone().filter(|s| !s.trim().is_empty());
        let language = pick(&binding.language);
        let model = pick(&binding.model);
        if language.is_none() && model.is_none() {
            return None;
        }
        Some(Self { language, model })
    }

    /// `settings` as this recording sees them.
    pub fn apply(&self, settings: &mut AppSettings) {
        if let Some(language) = &self.language {
            settings.selected_language = language.clone();
        }
        if let Some(model) = &self.model {
            settings.selected_model = model.clone();
        }
    }
}

/// Whether a dictation that wants `wanted` must load it. A file job holds
/// whatever model it runs on, so a dictation then uses that model as is.
pub fn needs_load(
    loaded: Option<&str>,
    wanted: &str,
    reload_pending: bool,
    job_active: bool,
) -> bool {
    if job_active && loaded.is_some() {
        return false;
    }
    reload_pending || loaded != Some(wanted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{get_default_settings, ShortcutBinding};

    fn with_binding(language: Option<&str>, model: Option<&str>) -> AppSettings {
        let mut settings = get_default_settings();
        settings.selected_language = "en".to_string();
        settings.selected_model = "parakeet-unified-en-0.6b".to_string();
        settings.bindings.insert(
            "transcribe_lang_1".to_string(),
            ShortcutBinding {
                id: "transcribe_lang_1".to_string(),
                name: "Transcribe".to_string(),
                description: String::new(),
                default_binding: String::new(),
                current_binding: "ctrl+option+r".to_string(),
                language: language.map(str::to_string),
                model: model.map(str::to_string),
            },
        );
        settings
    }

    #[test]
    fn override_applies_to_a_copy_and_never_to_saved_settings() {
        let saved = with_binding(Some("ru"), Some("whisper-large-v3-turbo"));
        let o = RecordingOverride::for_binding(&saved, "transcribe_lang_1").unwrap();

        let mut for_recording = saved.clone();
        o.apply(&mut for_recording);

        assert_eq!(for_recording.selected_language, "ru");
        assert_eq!(for_recording.selected_model, "whisper-large-v3-turbo");
        assert_eq!(saved.selected_language, "en");
        assert_eq!(saved.selected_model, "parakeet-unified-en-0.6b");
    }

    #[test]
    fn language_only_override_keeps_the_global_model() {
        let saved = with_binding(Some("he"), None);
        let o = RecordingOverride::for_binding(&saved, "transcribe_lang_1").unwrap();
        let mut s = saved.clone();
        o.apply(&mut s);
        assert_eq!(s.selected_language, "he");
        assert_eq!(s.selected_model, "parakeet-unified-en-0.6b");
    }

    #[test]
    fn builtin_and_empty_bindings_carry_no_override() {
        let saved = with_binding(Some(""), Some("  "));
        assert_eq!(RecordingOverride::for_binding(&saved, "transcribe"), None);
        assert_eq!(
            RecordingOverride::for_binding(&saved, "transcribe_lang_1"),
            None
        );
        assert_eq!(RecordingOverride::for_binding(&saved, "missing"), None);
    }

    #[test]
    fn a_loaded_model_is_kept_only_when_it_is_the_one_wanted() {
        assert!(!needs_load(Some("a"), "a", false, false));
        assert!(needs_load(Some("a"), "b", false, false));
        assert!(needs_load(None, "a", false, false));
        assert!(needs_load(Some("a"), "a", true, false));
    }

    #[test]
    fn a_file_job_keeps_its_model() {
        assert!(!needs_load(Some("a"), "b", false, true));
        assert!(!needs_load(Some("a"), "b", true, true));
        // Nothing loaded: the dictation still needs a model.
        assert!(needs_load(None, "b", false, true));
    }
}
