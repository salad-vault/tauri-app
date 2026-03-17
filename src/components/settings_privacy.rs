use leptos::prelude::*;

use crate::i18n::{t, Language};
use crate::components::settings::UserSettings;

#[component]
pub fn SettingsPrivacy(
    settings: ReadSignal<UserSettings>,
    set_settings: WriteSignal<UserSettings>,
    on_save: impl Fn() + Clone + Send + 'static,
) -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();
    let save = on_save.clone();

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">{move || t("privacy.section_title", lang.get())}</h2>
            <p class="settings-section-desc">{move || t("privacy.section_desc", lang.get())}</p>

            <div class="settings-group">
                <h3>{move || t("privacy.crash_reports_title", lang.get())}</h3>
                <div class="settings-row">
                    <label>{move || t("privacy.crash_reports", lang.get())}</label>
                    <input
                        type="checkbox"
                        class="settings-toggle"
                        prop:checked=move || settings.get().crash_reports
                        on:change={
                            let save = save.clone();
                            move |ev| {
                                let checked = event_target_checked(&ev);
                                let mut s = settings.get_untracked();
                                s.crash_reports = checked;
                                set_settings.set(s);
                                save();
                            }
                        }
                    />
                </div>
                <p class="settings-hint">{move || t("privacy.crash_reports_hint", lang.get())}</p>
            </div>
        </div>
    }
}
