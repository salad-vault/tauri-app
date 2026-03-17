use leptos::prelude::*;

use crate::i18n::{t, Language};

#[component]
pub fn Documentation(
    on_back: Callback<()>,
) -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();
    let (open_section, set_open_section) = signal(Option::<u32>::None);

    let toggle = move |id: u32| {
        set_open_section.set(if open_section.get() == Some(id) { None } else { Some(id) });
    };

    view! {
        <div class="documentation-page">
            <header class="settings-header">
                <div class="header-left">
                    <button class="btn btn-ghost" on:click=move |_| on_back.run(())>
                        {move || t("back", lang.get())}
                    </button>
                    <h1>{move || t("doc.title", lang.get())}</h1>
                </div>
            </header>

            <div class="doc-content">
                <p class="doc-intro">{move || t("doc.intro", lang.get())}</p>

                // ── Section 1: What is SaladVault? ──
                <div class="doc-section">
                    <button class="doc-section-header" on:click=move |_| toggle(1)>
                        <span class="doc-section-icon">"🥗"</span>
                        <span class="doc-section-title">{move || t("doc.s1.title", lang.get())}</span>
                        <span class="doc-chevron">{move || if open_section.get() == Some(1) { "▾" } else { "▸" }}</span>
                    </button>
                    {move || {
                        if open_section.get() == Some(1) {
                            view! {
                                <div class="doc-section-body">
                                    <p>{move || t("doc.s1.p1", lang.get())}</p>
                                    <p>{move || t("doc.s1.p2", lang.get())}</p>
                                    <p>{move || t("doc.s1.p3", lang.get())}</p>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                </div>

                // ── Section 2: Master Password ──
                <div class="doc-section">
                    <button class="doc-section-header" on:click=move |_| toggle(2)>
                        <span class="doc-section-icon">"🔑"</span>
                        <span class="doc-section-title">{move || t("doc.s2.title", lang.get())}</span>
                        <span class="doc-chevron">{move || if open_section.get() == Some(2) { "▾" } else { "▸" }}</span>
                    </button>
                    {move || {
                        if open_section.get() == Some(2) {
                            view! {
                                <div class="doc-section-body">
                                    <p>{move || t("doc.s2.p1", lang.get())}</p>
                                    <p>{move || t("doc.s2.p2", lang.get())}</p>
                                    <div class="doc-tip">
                                        <span class="doc-tip-icon">"💡"</span>
                                        <p>{move || t("doc.s2.tip", lang.get())}</p>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                </div>

                // ── Section 3: Device Key ──
                <div class="doc-section">
                    <button class="doc-section-header" on:click=move |_| toggle(3)>
                        <span class="doc-section-icon">"💾"</span>
                        <span class="doc-section-title">{move || t("doc.s3.title", lang.get())}</span>
                        <span class="doc-chevron">{move || if open_section.get() == Some(3) { "▾" } else { "▸" }}</span>
                    </button>
                    {move || {
                        if open_section.get() == Some(3) {
                            view! {
                                <div class="doc-section-body">
                                    <p>{move || t("doc.s3.p1", lang.get())}</p>
                                    <p>{move || t("doc.s3.p2", lang.get())}</p>
                                    <div class="doc-warning">
                                        <span class="doc-tip-icon">"⚠️"</span>
                                        <p>{move || t("doc.s3.warning", lang.get())}</p>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                </div>

                // ── Section 4: Dual-Lock ──
                <div class="doc-section">
                    <button class="doc-section-header" on:click=move |_| toggle(4)>
                        <span class="doc-section-icon">"🔐"</span>
                        <span class="doc-section-title">{move || t("doc.s4.title", lang.get())}</span>
                        <span class="doc-chevron">{move || if open_section.get() == Some(4) { "▾" } else { "▸" }}</span>
                    </button>
                    {move || {
                        if open_section.get() == Some(4) {
                            view! {
                                <div class="doc-section-body">
                                    <p>{move || t("doc.s4.p1", lang.get())}</p>
                                    <p>{move || t("doc.s4.p2", lang.get())}</p>
                                    <p>{move || t("doc.s4.p3", lang.get())}</p>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                </div>

                // ── Section 5: Saladiers & Feuilles ──
                <div class="doc-section">
                    <button class="doc-section-header" on:click=move |_| toggle(5)>
                        <span class="doc-section-icon">"📂"</span>
                        <span class="doc-section-title">{move || t("doc.s5.title", lang.get())}</span>
                        <span class="doc-chevron">{move || if open_section.get() == Some(5) { "▾" } else { "▸" }}</span>
                    </button>
                    {move || {
                        if open_section.get() == Some(5) {
                            view! {
                                <div class="doc-section-body">
                                    <p>{move || t("doc.s5.p1", lang.get())}</p>
                                    <p>{move || t("doc.s5.p2", lang.get())}</p>
                                    <p>{move || t("doc.s5.p3", lang.get())}</p>
                                    <div class="doc-tip">
                                        <span class="doc-tip-icon">"🕵️"</span>
                                        <p>{move || t("doc.s5.tip", lang.get())}</p>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                </div>

                // ── Section 6: Recovery Kit ──
                <div class="doc-section">
                    <button class="doc-section-header" on:click=move |_| toggle(6)>
                        <span class="doc-section-icon">"🚨"</span>
                        <span class="doc-section-title">{move || t("doc.s6.title", lang.get())}</span>
                        <span class="doc-chevron">{move || if open_section.get() == Some(6) { "▾" } else { "▸" }}</span>
                    </button>
                    {move || {
                        if open_section.get() == Some(6) {
                            view! {
                                <div class="doc-section-body">
                                    <p>{move || t("doc.s6.p1", lang.get())}</p>
                                    <p>{move || t("doc.s6.p2", lang.get())}</p>
                                    <div class="doc-warning">
                                        <span class="doc-tip-icon">"📝"</span>
                                        <p>{move || t("doc.s6.tip", lang.get())}</p>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                </div>

                // ── Section 7: Safety Features ──
                <div class="doc-section">
                    <button class="doc-section-header" on:click=move |_| toggle(7)>
                        <span class="doc-section-icon">"🛡️"</span>
                        <span class="doc-section-title">{move || t("doc.s7.title", lang.get())}</span>
                        <span class="doc-chevron">{move || if open_section.get() == Some(7) { "▾" } else { "▸" }}</span>
                    </button>
                    {move || {
                        if open_section.get() == Some(7) {
                            view! {
                                <div class="doc-section-body">
                                    <p>{move || t("doc.s7.p1", lang.get())}</p>
                                    <ul class="doc-list">
                                        <li>{move || t("doc.s7.feat1", lang.get())}</li>
                                        <li>{move || t("doc.s7.feat2", lang.get())}</li>
                                        <li>{move || t("doc.s7.feat3", lang.get())}</li>
                                        <li>{move || t("doc.s7.feat4", lang.get())}</li>
                                    </ul>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                </div>

                // ── Section 8: FAQ ──
                <div class="doc-section">
                    <button class="doc-section-header" on:click=move |_| toggle(8)>
                        <span class="doc-section-icon">"❓"</span>
                        <span class="doc-section-title">{move || t("doc.s8.title", lang.get())}</span>
                        <span class="doc-chevron">{move || if open_section.get() == Some(8) { "▾" } else { "▸" }}</span>
                    </button>
                    {move || {
                        if open_section.get() == Some(8) {
                            view! {
                                <div class="doc-section-body">
                                    <div class="doc-faq">
                                        <h4>{move || t("doc.s8.q1", lang.get())}</h4>
                                        <p>{move || t("doc.s8.a1", lang.get())}</p>
                                    </div>
                                    <div class="doc-faq">
                                        <h4>{move || t("doc.s8.q2", lang.get())}</h4>
                                        <p>{move || t("doc.s8.a2", lang.get())}</p>
                                    </div>
                                    <div class="doc-faq">
                                        <h4>{move || t("doc.s8.q3", lang.get())}</h4>
                                        <p>{move || t("doc.s8.a3", lang.get())}</p>
                                    </div>
                                    <div class="doc-faq">
                                        <h4>{move || t("doc.s8.q4", lang.get())}</h4>
                                        <p>{move || t("doc.s8.a4", lang.get())}</p>
                                    </div>
                                    <div class="doc-faq">
                                        <h4>{move || t("doc.s8.q5", lang.get())}</h4>
                                        <p>{move || t("doc.s8.a5", lang.get())}</p>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
