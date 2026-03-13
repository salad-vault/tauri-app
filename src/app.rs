use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::dashboard::Dashboard;
use crate::components::login::Login;
use crate::components::nag_screen::NagScreen;
use crate::components::panic_unlock::PanicUnlock;
use crate::components::recovery::Recovery;
use crate::components::register::Register;
use crate::components::saladier_view::SaladierView;
use crate::components::settings::{AutoLockTimeout, Settings, Theme, UserSettings};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// The application's current view state.
#[derive(Clone, Debug, PartialEq)]
enum AppView {
    Login,
    Register,
    NagScreen,
    Dashboard,
    SaladierUnlock { uuid: String, name: String },
    SaladierView { uuid: String, name: String },
    Recovery,
    Settings,
}

/// Apply theme by setting data-theme attribute on <html>.
fn apply_theme(theme: &Theme) {
    let theme_str = match theme {
        Theme::Dark => "dark",
        Theme::Light => "light",
    };
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if let Some(el) = doc.document_element() {
                let _ = el.set_attribute("data-theme", theme_str);
            }
        }
    }
}

/// Apply screenshot protection via Tauri command.
fn apply_screenshot_protection(enabled: bool) {
    spawn_local(async move {
        #[derive(Serialize)]
        struct Args {
            enabled: bool,
        }
        let args = serde_wasm_bindgen::to_value(&Args { enabled }).unwrap();
        let _ = invoke("apply_screenshot_protection", args).await;
    });
}

/// Navigate to Dashboard or NagScreen depending on recovery status.
/// Also loads settings and applies screenshot preference.
fn check_recovery_and_navigate(
    set_current_view: WriteSignal<AppView>,
    set_user_settings: WriteSignal<Option<UserSettings>>,
) {
    spawn_local(async move {
        // Load settings and apply screenshot preference
        let settings_args = serde_wasm_bindgen::to_value(&()).unwrap();
        if let Ok(settings_result) = invoke("get_settings", settings_args).await {
            if let Ok(s) = serde_wasm_bindgen::from_value::<UserSettings>(settings_result) {
                apply_theme(&s.theme);
                if !s.screenshot_protection {
                    apply_screenshot_protection(false);
                }
                set_user_settings.set(Some(s));
            }
        }

        let args = serde_wasm_bindgen::to_value(&()).unwrap();
        if let Ok(result) = invoke("check_recovery_status", args).await {
            let confirmed = result.as_bool().unwrap_or(false);
            if confirmed {
                set_current_view.set(AppView::Dashboard);
            } else {
                set_current_view.set(AppView::NagScreen);
            }
        } else {
            set_current_view.set(AppView::NagScreen);
        }

        // Send heartbeat to server if connected (for Dead Man's Switch)
        let hb_args = serde_wasm_bindgen::to_value(&()).unwrap();
        let _ = invoke("deadman_heartbeat", hb_args).await;
    });
}

#[component]
pub fn App() -> impl IntoView {
    let (current_view, set_current_view) = signal(AppView::Login);

    // User settings (loaded after login for auto-lock + screenshot protection)
    let (user_settings, set_user_settings) = signal(Option::<UserSettings>::None);

    // Auto-lock: Interval handle + event listener closures
    let interval_handle: Rc<RefCell<Option<gloo_timers::callback::Interval>>> =
        Rc::new(RefCell::new(None));
    let listeners: Rc<RefCell<Vec<(String, wasm_bindgen::closure::Closure<dyn Fn()>)>>> =
        Rc::new(RefCell::new(Vec::new()));

    // ── Callbacks for child components ──
    // Direct navigation callbacks replace the previous boolean signal + Effect
    // intermediaries, eliminating the "set true → effect resets to false" anti-pattern.

    let on_login = Callback::new(move |_: ()| {
        check_recovery_and_navigate(set_current_view, set_user_settings);
    });

    let on_switch_register = Callback::new(move |_: ()| {
        set_current_view.set(AppView::Register);
    });

    let on_switch_login = Callback::new(move |_: ()| {
        set_current_view.set(AppView::Login);
    });

    let on_registered = Callback::new(move |_: ()| {
        set_current_view.set(AppView::NagScreen);
    });

    let on_nag_confirmed = Callback::new(move |_: ()| {
        set_current_view.set(AppView::Dashboard);
    });

    let on_select_saladier = Callback::new(move |(uuid, name): (String, String)| {
        set_current_view.set(AppView::SaladierUnlock { uuid, name });
    });

    let on_saladier_unlocked = Callback::new(move |_: ()| {
        if let AppView::SaladierUnlock { uuid, name } = current_view.get_untracked() {
            set_current_view.set(AppView::SaladierView { uuid, name });
        }
    });

    let on_saladier_cancel = Callback::new(move |_: ()| {
        set_current_view.set(AppView::Dashboard);
    });

    let on_back_to_dashboard = Callback::new(move |_: ()| {
        set_current_view.set(AppView::Dashboard);
    });

    let on_show_recovery = Callback::new(move |_: ()| {
        set_current_view.set(AppView::Recovery);
    });

    let on_close_recovery = Callback::new(move |_: ()| {
        set_current_view.set(AppView::Dashboard);
    });

    let on_show_settings = Callback::new(move |_: ()| {
        set_current_view.set(AppView::Settings);
    });

    let on_settings_back = Callback::new(move |_: ()| {
        set_current_view.set(AppView::Dashboard);
    });

    // Logout: kept as signal + Effect because cleanup captures non-Send Rc types
    let (on_logout, set_on_logout) = signal(false);
    {
        let interval_handle = interval_handle.clone();
        let listeners = listeners.clone();
        Effect::new(move |_| {
            if on_logout.get() {
                set_on_logout.set(false);
                // Cleanup auto-lock
                *interval_handle.borrow_mut() = None; // drops Interval, stopping it
                {
                    let window = web_sys::window().unwrap();
                    let document = window.document().unwrap();
                    let mut ls = listeners.borrow_mut();
                    for (event, closure) in ls.drain(..) {
                        if event == "visibilitychange" {
                            let _ = document.remove_event_listener_with_callback(
                                &event,
                                closure.as_ref().unchecked_ref(),
                            );
                        } else {
                            let _ = window.remove_event_listener_with_callback(
                                &event,
                                closure.as_ref().unchecked_ref(),
                            );
                        }
                    }
                }
                set_user_settings.set(None);

                // Reset theme to dark on logout (default for login screen)
                apply_theme(&Theme::Dark);

                // Re-enable screenshot protection on logout (security default)
                apply_screenshot_protection(true);

                // Call lock command
                spawn_local(async {
                    let args = serde_wasm_bindgen::to_value(&()).unwrap();
                    let _ = invoke("lock", args).await;
                });

                set_current_view.set(AppView::Login);
            }
        });
    }

    // Apply theme reactively when user_settings change (e.g. toggle in Settings)
    Effect::new(move |_| {
        if let Some(ref s) = user_settings.get() {
            apply_theme(&s.theme);
        }
    });

    // Start auto-lock polling when settings are loaded
    {
        let interval_handle = interval_handle.clone();
        let listeners = listeners.clone();
        Effect::new(move |_| {
            let settings_opt = user_settings.get();
            if let Some(ref settings) = settings_opt {
                // --- Activity listeners (throttled) ---
                let last_update = Rc::new(RefCell::new(js_sys::Date::now()));
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();

                // Throttled activity update: max 1 call per 5s
                let make_activity_cb = {
                    let last_update = last_update.clone();
                    move || {
                        let now = js_sys::Date::now();
                        let last = *last_update.borrow();
                        if now - last > 5_000.0 {
                            *last_update.borrow_mut() = now;
                            spawn_local(async {
                                let args = serde_wasm_bindgen::to_value(&()).unwrap();
                                let _ = invoke("update_last_activity", args).await;
                            });
                        }
                    }
                };

                // Attach mousemove listener
                {
                    let cb = make_activity_cb.clone();
                    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                        cb();
                    }) as Box<dyn Fn()>);
                    let _ = window.add_event_listener_with_callback(
                        "mousemove",
                        closure.as_ref().unchecked_ref(),
                    );
                    listeners.borrow_mut().push(("mousemove".to_string(), closure));
                }

                // Attach keydown listener
                {
                    let cb = make_activity_cb.clone();
                    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                        cb();
                    }) as Box<dyn Fn()>);
                    let _ = window.add_event_listener_with_callback(
                        "keydown",
                        closure.as_ref().unchecked_ref(),
                    );
                    listeners.borrow_mut().push(("keydown".to_string(), closure));
                }

                // Visibility change listener (proxy for sleep / tab switch / Immediate lock)
                let is_immediate = settings.auto_lock_timeout == AutoLockTimeout::Immediate;
                if settings.auto_lock_on_sleep || is_immediate {
                    let set_view = set_current_view;
                    let pending_lock: Rc<RefCell<Option<gloo_timers::callback::Timeout>>> =
                        Rc::new(RefCell::new(None));
                    let pending_lock_inner = pending_lock.clone();
                    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                        let doc = web_sys::window().unwrap().document().unwrap();
                        let hidden = js_sys::Reflect::get(&doc, &"hidden".into())
                            .unwrap()
                            .as_bool()
                            .unwrap_or(false);
                        if hidden {
                            let set_view = set_view;
                            let do_lock = move || {
                                spawn_local(async move {
                                    let args = serde_wasm_bindgen::to_value(&()).unwrap();
                                    let _ = invoke("lock", args).await;
                                });
                                apply_screenshot_protection(true);
                                set_view.set(AppView::Login);
                            };
                            if is_immediate {
                                // Immediate: lock with no grace period
                                do_lock();
                            } else {
                                // Grace period: 3 seconds before locking
                                let timeout = gloo_timers::callback::Timeout::new(3_000, do_lock);
                                *pending_lock_inner.borrow_mut() = Some(timeout);
                            }
                        } else {
                            // Window became visible again: cancel pending lock
                            *pending_lock_inner.borrow_mut() = None;
                        }
                    }) as Box<dyn Fn()>);
                    let _ = document.add_event_listener_with_callback(
                        "visibilitychange",
                        closure.as_ref().unchecked_ref(),
                    );
                    listeners
                        .borrow_mut()
                        .push(("visibilitychange".to_string(), closure));
                }

                // --- Inactivity polling ---
                // Immediate uses visibilitychange (above), not polling
                let timeout_secs = match settings.auto_lock_timeout {
                    AutoLockTimeout::Immediate => None,
                    AutoLockTimeout::After1Min => Some(60u64),
                    AutoLockTimeout::After5Min => Some(300),
                    AutoLockTimeout::Never => None,
                };

                if let Some(max_secs) = timeout_secs {
                    if settings.auto_lock_on_inactivity {
                        let set_view = set_current_view;
                        let interval = gloo_timers::callback::Interval::new(10_000, move || {
                            let set_view = set_view;
                            spawn_local(async move {
                                let args = serde_wasm_bindgen::to_value(&()).unwrap();
                                if let Ok(result) = invoke("get_inactivity_seconds", args).await {
                                    if let Some(secs) = result.as_f64() {
                                        if secs as u64 >= max_secs {
                                            // Lock the app
                                            let lock_args =
                                                serde_wasm_bindgen::to_value(&()).unwrap();
                                            let _ = invoke("lock", lock_args).await;
                                            apply_screenshot_protection(true);
                                            set_view.set(AppView::Login);
                                        }
                                    }
                                }
                            });
                        });
                        *interval_handle.borrow_mut() = Some(interval);
                    }
                }
            }
        });
    }

    view! {
        <div class="app">
            {move || {
                match current_view.get() {
                    AppView::Login => {
                        view! {
                            <Login
                                on_login=on_login
                                on_switch_register=on_switch_register
                            />
                        }.into_any()
                    }
                    AppView::Register => {
                        view! {
                            <Register
                                on_registered=on_registered
                                on_switch_login=on_switch_login
                            />
                        }.into_any()
                    }
                    AppView::NagScreen => {
                        view! {
                            <NagScreen on_confirmed=on_nag_confirmed />
                        }.into_any()
                    }
                    AppView::Dashboard => {
                        view! {
                            <Dashboard
                                on_select_saladier=on_select_saladier
                                on_logout=set_on_logout
                                on_show_recovery=on_show_recovery
                                on_show_settings=on_show_settings
                            />
                        }.into_any()
                    }
                    AppView::SaladierUnlock { uuid, name } => {
                        view! {
                            <PanicUnlock
                                saladier_uuid=uuid
                                saladier_name=name
                                on_unlocked=on_saladier_unlocked
                                on_cancel=on_saladier_cancel
                            />
                        }.into_any()
                    }
                    AppView::SaladierView { uuid, name } => {
                        view! {
                            <SaladierView
                                saladier_uuid=uuid
                                saladier_name=name
                                on_back=on_back_to_dashboard
                            />
                        }.into_any()
                    }
                    AppView::Recovery => {
                        view! {
                            <Recovery on_close=on_close_recovery />
                        }.into_any()
                    }
                    AppView::Settings => {
                        view! {
                            <Settings on_back=on_settings_back />
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}
