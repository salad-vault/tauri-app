use leptos::prelude::*;

use crate::components::settings::UserSettings;

#[component]
pub fn SettingsPrivacy(
    settings: ReadSignal<UserSettings>,
    set_settings: WriteSignal<UserSettings>,
    on_save: impl Fn() + Clone + Send + 'static,
) -> impl IntoView {
    let save = on_save.clone();

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">"🌐 Vie Privée & Réseau"</h2>
            <p class="settings-section-desc">"Contrôlez les fuites de métadonnées et les communications réseau."</p>

            <div class="settings-group">
                <h3>"Rapports de crash"</h3>
                <div class="settings-row">
                    <label>"Envoyer des rapports d'erreurs anonymes"</label>
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
                <p class="settings-hint">"Opt-in uniquement. Aucune donnée personnelle n'est incluse dans les rapports."</p>
            </div>
        </div>
    }
}
