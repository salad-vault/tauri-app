mod en;
mod fr;

/// Supported languages.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Language {
    #[default]
    Fr,
    En,
}

/// Load the language preference from localStorage.
pub fn load_language() -> Language {
    let window = web_sys::window().unwrap();
    if let Ok(Some(storage)) = window.local_storage() {
        if let Ok(Some(val)) = storage.get_item("sv_lang") {
            if val == "en" {
                return Language::En;
            }
        }
    }
    Language::Fr
}

/// Save the language preference to localStorage.
pub fn save_language(lang: Language) {
    let window = web_sys::window().unwrap();
    if let Ok(Some(storage)) = window.local_storage() {
        let _ = storage.set_item(
            "sv_lang",
            match lang {
                Language::Fr => "fr",
                Language::En => "en",
            },
        );
    }
}

/// Translate a key for the given language.
/// Returns the key itself if no translation exists (fallback).
pub fn t(key: &'static str, lang: Language) -> &'static str {
    let lookup = match lang {
        Language::Fr => fr::lookup(key),
        Language::En => en::lookup(key),
    };
    lookup.unwrap_or(key)
}

/// Dynamic translation helper for strings that need runtime formatting.
/// Use `t()` for static strings; use this for strings with placeholders.
pub fn tf(key: &'static str, lang: Language, args: &[&str]) -> String {
    use Language::*;
    match (key, lang) {
        ("panic.remaining", Fr) => format!(
            "Mot de passe incorrect. {} tentative(s) restante(s).",
            args.first().unwrap_or(&"?")
        ),
        ("panic.remaining", En) => format!(
            "Incorrect password. {} attempt(s) remaining.",
            args.first().unwrap_or(&"?")
        ),
        ("recovery.error_fmt", Fr) => format!("Erreur : {}", args.first().unwrap_or(&"")),
        ("recovery.error_fmt", En) => format!("Error: {}", args.first().unwrap_or(&"")),
        _ => t(key, lang).to_string(),
    }
}
