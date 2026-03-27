use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::i18n::{t, Language};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI_PLUGIN_OPENER__"], js_name = openUrl, catch)]
    async fn open_url(url: &str) -> Result<JsValue, JsValue>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SubscriptionStatus {
    plan: String,
    status: String,
    trial_end: Option<String>,
    current_period_end: Option<String>,
}

#[component]
pub fn SettingsSubscription() -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();
    let (sub_status, set_sub_status) = signal::<Option<SubscriptionStatus>>(None);
    let (loading, set_loading) = signal(false);
    let (error_msg, set_error_msg) = signal(String::new());
    let (connected, set_connected) = signal(false);

    // Check server connection and fetch subscription status on mount
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            let args = serde_json::json!({});
            let args_js = serde_wasm_bindgen::to_value(&args).unwrap_or(JsValue::NULL);
            if let Ok(result) = invoke("server_is_connected", args_js).await {
                if let Some(is_connected) = result.as_bool() {
                    set_connected.set(is_connected);
                    if is_connected {
                        let args2 = serde_json::json!({});
                        let args_js2 = serde_wasm_bindgen::to_value(&args2).unwrap_or(JsValue::NULL);
                        if let Ok(status) = invoke("subscription_status", args_js2).await {
                            if let Ok(s) = serde_wasm_bindgen::from_value::<SubscriptionStatus>(status) {
                                set_sub_status.set(Some(s));
                            }
                        }
                    }
                }
            }
        });
    });

    let is_maraicher = move || {
        sub_status.get().as_ref().map_or(false, |s| s.plan == "maraicher" && (s.status == "active" || s.status == "trialing"))
    };

    let current_plan_name = move || {
        sub_status.get().as_ref().map_or(
            t("sub.tier_jardinier", lang.get()),
            |s| match s.plan.as_str() {
                "maraicher" if s.status == "active" || s.status == "trialing" => t("sub.tier_maraicher", lang.get()),
                _ => t("sub.tier_jardinier", lang.get()),
            },
        )
    };

    let current_plan_price = move || {
        if is_maraicher() {
            format!("{} {}", t("sub.price_maraicher", lang.get()), t("sub.per_month", lang.get()))
        } else {
            t("sub.free", lang.get()).to_string()
        }
    };

    let plan_badge = move || {
        let status = sub_status.get();
        match status.as_ref() {
            Some(s) if s.status == "trialing" => t("sub.trial_badge", lang.get()),
            _ => t("sub.current_plan", lang.get()),
        }
    };

    let on_checkout = move |_| {
        set_loading.set(true);
        set_error_msg.set(String::new());
        leptos::task::spawn_local(async move {
            let args = serde_json::json!({});
            let args_js = serde_wasm_bindgen::to_value(&args).unwrap_or(JsValue::NULL);
            match invoke("subscription_checkout", args_js).await {
                Ok(url_js) => {
                    if let Some(url) = url_js.as_string() {
                        let _ = open_url(&url).await;
                    }
                }
                Err(e) => {
                    let msg = e.as_string().unwrap_or_else(|| "Erreur lors de la creation de la session".to_string());
                    set_error_msg.set(msg);
                }
            }
            set_loading.set(false);
        });
    };

    let on_portal = move |_| {
        set_loading.set(true);
        set_error_msg.set(String::new());
        leptos::task::spawn_local(async move {
            let args = serde_json::json!({});
            let args_js = serde_wasm_bindgen::to_value(&args).unwrap_or(JsValue::NULL);
            match invoke("subscription_portal", args_js).await {
                Ok(url_js) => {
                    if let Some(url) = url_js.as_string() {
                        let _ = open_url(&url).await;
                    }
                }
                Err(e) => {
                    let msg = e.as_string().unwrap_or_else(|| "Erreur lors de l'ouverture du portail".to_string());
                    set_error_msg.set(msg);
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">{move || t("sub.title", lang.get())}</h2>

            // Current plan banner
            <div class="subscription-current-plan">
                <div class="subscription-current-info">
                    <span class="subscription-plan-name">{current_plan_name}</span>
                    <span class="subscription-plan-price">{current_plan_price}</span>
                </div>
                <span class="pricing-badge pricing-badge-active">{plan_badge}</span>
            </div>

            // Error message
            <Show when=move || !error_msg.get().is_empty()>
                <div class="settings-error">{move || error_msg.get()}</div>
            </Show>

            // 4-column pricing grid
            <div class="pricing-grid">
                // Card 1 — Jardinier (free)
                <div class="pricing-card">
                    <div class="pricing-card-header">
                        <div class="pricing-tier-name">{move || t("sub.tier_jardinier", lang.get())}</div>
                        <div class="pricing-tier-price">
                            <span class="pricing-amount">{move || t("sub.free", lang.get())}</span>
                        </div>
                        <div class="pricing-tier-subtitle">{move || t("sub.jardinier_subtitle", lang.get())}</div>
                    </div>

                    <ul class="pricing-features">
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_0knowledge", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_dual_lock", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_unlimited_local", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_password_gen", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_recovery_kit", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_import_export", lang.get())}</span>
                        </li>
                    </ul>

                    <Show when=move || !is_maraicher() fallback=|| view! {}>
                        <button class="pricing-btn pricing-btn-current">{move || t("sub.current_plan", lang.get())}</button>
                    </Show>
                </div>

                // Card 2 — Maraicher Pro (recommended)
                <div class="pricing-card pricing-card-highlighted">
                    <span class="pricing-badge pricing-badge-recommended">{move || t("sub.recommended", lang.get())}</span>
                    <div class="pricing-card-header">
                        <div class="pricing-tier-name">{move || t("sub.tier_maraicher", lang.get())}</div>
                        <div class="pricing-tier-price">
                            <span class="pricing-amount">{move || t("sub.price_maraicher", lang.get())}</span>
                            <span class="pricing-period">{move || t("sub.per_month", lang.get())}</span>
                        </div>
                        <div class="pricing-tier-subtitle">{move || t("sub.maraicher_subtitle", lang.get())}</div>
                    </div>

                    <div class="pricing-divider">{move || t("sub.all_jardinier_plus", lang.get())}</div>

                    <ul class="pricing-features">
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_cloud_sync", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_multi_device", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_shared_vaults", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_priority_support", lang.get())}</span>
                        </li>
                    </ul>

                    <Show when=move || is_maraicher()>
                        <button class="pricing-btn pricing-btn-current">{move || t("sub.current_plan", lang.get())}</button>
                        <button
                            class="pricing-btn pricing-btn-secondary"
                            on:click=on_portal
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() { t("sub.loading", lang.get()) } else { t("sub.manage_subscription", lang.get()) }}
                        </button>
                    </Show>
                    <Show when=move || !is_maraicher() && connected.get()>
                        <button
                            class="pricing-btn pricing-btn-upgrade"
                            on:click=on_checkout
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() { t("sub.loading", lang.get()) } else { t("sub.upgrade_maraicher", lang.get()) }}
                        </button>
                    </Show>
                    <Show when=move || !is_maraicher() && !connected.get()>
                        <button class="pricing-btn pricing-btn-soon">{move || t("sub.connect_first", lang.get())}</button>
                    </Show>
                </div>

                // Card 3 — Potager Familial
                <div class="pricing-card">
                    <div class="pricing-card-header">
                        <div class="pricing-tier-name">{move || t("sub.tier_familial", lang.get())}</div>
                        <div class="pricing-tier-price">
                            <span class="pricing-amount">{move || t("sub.price_familial", lang.get())}</span>
                            <span class="pricing-period">{move || t("sub.per_month", lang.get())}</span>
                        </div>
                        <div class="pricing-tier-subtitle">{move || t("sub.familial_subtitle", lang.get())}</div>
                    </div>

                    <div class="pricing-divider">{move || t("sub.all_maraicher_plus", lang.get())}</div>

                    <ul class="pricing-features">
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_family_sharing", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_shared_vaults", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_admin_console", lang.get())}</span>
                        </li>
                    </ul>

                    <button class="pricing-btn pricing-btn-soon">{move || t("sub.coming_soon", lang.get())}</button>
                </div>

                // Card 4 — Exploitation (enterprise)
                <div class="pricing-card">
                    <div class="pricing-card-header">
                        <div class="pricing-tier-name">{move || t("sub.tier_enterprise", lang.get())}</div>
                        <div class="pricing-tier-price">
                            <span class="pricing-amount">{move || t("sub.price_enterprise", lang.get())}</span>
                        </div>
                        <div class="pricing-tier-subtitle">{move || t("sub.enterprise_subtitle", lang.get())}</div>
                    </div>

                    <div class="pricing-divider">{move || t("sub.all_familial_plus", lang.get())}</div>

                    <ul class="pricing-features">
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_unlimited_users", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_sso", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"+"</span>
                            <span>{move || t("sub.feat_audit_logs", lang.get())}</span>
                        </li>
                    </ul>

                    <button class="pricing-btn pricing-btn-soon">{move || t("sub.coming_soon", lang.get())}</button>
                </div>
            </div>

            // Footer note
            <div class="subscription-footer-note">
                <p>{move || t("sub.footer_note", lang.get())}</p>
            </div>
        </div>
    }
}
