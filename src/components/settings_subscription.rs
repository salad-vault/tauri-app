use leptos::prelude::*;

use crate::i18n::{t, Language};

#[component]
pub fn SettingsSubscription() -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">{move || t("sub.title", lang.get())}</h2>

            // Current plan banner
            <div class="subscription-current-plan">
                <div class="subscription-current-info">
                    <span class="subscription-plan-name">{move || t("sub.tier_jardinier", lang.get())}</span>
                    <span class="subscription-plan-price">{move || t("sub.free", lang.get())}</span>
                </div>
                <span class="pricing-badge pricing-badge-active">{move || t("sub.current_plan", lang.get())}</span>
            </div>

            // 4-column pricing grid
            <div class="pricing-grid">
                // Card 1 — Jardinier (free, current)
                <div class="pricing-card">
                    <div class="pricing-card-header">
                        <span class="pricing-tier-icon">"🌱"</span>
                        <div class="pricing-tier-name">{move || t("sub.tier_jardinier", lang.get())}</div>
                        <div class="pricing-tier-price">
                            <span class="pricing-amount">{move || t("sub.free", lang.get())}</span>
                        </div>
                        <div class="pricing-tier-subtitle">{move || t("sub.jardinier_subtitle", lang.get())}</div>
                    </div>

                    <ul class="pricing-features">
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_zero_knowledge", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_dual_lock", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_unlimited_saladiers", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_password_gen", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_recovery_kit", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_autolock_screenshot", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_clipboard_clear", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_import_export", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_deadman_local", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_destroy_attempts", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_1_device", lang.get())}</span>
                        </li>
                    </ul>

                    <button class="pricing-btn pricing-btn-current">{move || t("sub.current_plan", lang.get())}</button>
                </div>

                // Card 2 — Maraîcher (recommended)
                <div class="pricing-card pricing-card-highlighted">
                    <span class="pricing-badge pricing-badge-recommended">{move || t("sub.recommended", lang.get())}</span>
                    <div class="pricing-card-header">
                        <span class="pricing-tier-icon">"🥬"</span>
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
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_sync_5_devices", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_browser_ext", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_sentinel", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_totp", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_secure_share", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_encrypted_attach", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_favorites_tags", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_custom_themes", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_priority_support", lang.get())}</span>
                        </li>
                    </ul>

                    <button class="pricing-btn pricing-btn-soon">{move || t("sub.coming_soon", lang.get())}</button>
                </div>

                // Card 3 — Potager Familial
                <div class="pricing-card">
                    <div class="pricing-card-header">
                        <span class="pricing-tier-icon">"👨\u{200d}👩\u{200d}👧\u{200d}👦"</span>
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
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_6_users", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_shared_saladiers", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_roles_permissions", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_admin_dashboard", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_unlimited_devices", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_deadman_server", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_digital_heritage", lang.get())}</span>
                        </li>
                    </ul>

                    <button class="pricing-btn pricing-btn-soon">{move || t("sub.coming_soon", lang.get())}</button>
                </div>

                // Card 4 — Exploitation (enterprise)
                <div class="pricing-card">
                    <div class="pricing-card-header">
                        <span class="pricing-tier-icon">"🏢"</span>
                        <div class="pricing-tier-name">{move || t("sub.tier_enterprise", lang.get())}</div>
                        <div class="pricing-tier-price">
                            <span class="pricing-amount">{move || t("sub.price_enterprise", lang.get())}</span>
                        </div>
                        <div class="pricing-tier-subtitle">{move || t("sub.enterprise_subtitle", lang.get())}</div>
                    </div>

                    <div class="pricing-divider">{move || t("sub.all_familial_plus", lang.get())}</div>

                    <ul class="pricing-features">
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_unlimited_users", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_sso", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_enterprise_policies", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_audit_log", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_groups_collections", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_scim", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_auto_rotation", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_self_hosted", lang.get())}</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>{move || t("sub.feat_sla_api", lang.get())}</span>
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
