use crate::server::{
    api::ApiError,
    auth::{Logout, Update, User},
};
use leptos::prelude::*;
use leptos_meta::Title;
use wasm_bindgen::JsCast;
use web_sys::{Event, FormData, HtmlFormElement, SubmitEvent};

#[component]
pub fn Dashboard(
    user: Resource<Result<Option<User>, ServerFnError>>,
    update: ServerAction<Update>,
    update_pfp: Action<FormData, Result<(), ServerFnError<ApiError>>, LocalStorage>,
    logout: ServerAction<Logout>,
) -> impl IntoView {
    let (show_name, set_show_name) = signal(false);
    let (show_avatar, set_show_avatar) = signal(false);
    view! {
        <Title text="Dashboard" />
        <div
            class="toner"
            class:hidden=move || !*show_name.read() && !*show_avatar.read()
            on:click=move |_| {
                set_show_name(false);
                set_show_avatar(false);
            }
        />
        <Username action=update show=show_name />
        <Avatar action=update_pfp show=show_avatar />
        <section id="dashboard">
            <Suspense fallback=|| view! { <h1>"Loading..."</h1> }>
                <h1>"Dashboard"</h1>
                <div class="section">
                    <h2>"Profile"</h2>
                    <div class="row">
                        <img
                            class="pfp"
                            src=move || {
                                format!(
                                    "/cdn/users/{}.jpg",
                                    user
                                        .get()
                                        .map(|res| {
                                            res.unwrap_or_default().unwrap_or_default().pfp
                                        })
                                        .unwrap_or("default.jpg".into()),
                                )
                            }
                            on:click=move |_| set_show_avatar(true)
                        />
                        <button class="primary" on:click=move |_| set_show_avatar(true)>
                            "Change Avatar"
                        </button>
                    </div>
                    <div class="row">
                        <div class="narrow">
                            <h3>"USERNAME"</h3>
                            <h4>
                                {move || {
                                    user.get()
                                        .map(|user| user.map(|user| user.map(|user| user.username)))
                                }}
                            </h4>
                        </div>
                        <button class="secondary" on:click=move |_| set_show_name(true)>
                            "Edit"
                        </button>
                    </div>
                    <div class="row">
                        <div class="narrow">
                            <h3>"PASSWORD"</h3>
                            <h4>"********"</h4>
                        </div>
                        <button class="secondary">"Edit"</button>
                    </div>
                    <h2>"Account"</h2>
                    <div>
                        <h4>"Log Out"</h4>
                        <p>"Logging out of your account will only affect this device."</p>
                        <button
                            class="primary"
                            on:click=move |_| {
                                let _ = logout.dispatch(Logout {});
                            }
                        >
                            "Log Out"
                        </button>
                    </div>
                    <div>
                        <h4>"Account Removal"</h4>
                        <p>
                            "Removing your account will anonymize your account and prevent"
                            "you from logging in. This action will not delete runs from the leaderboard."
                        </p>
                        <button class="secondary">"Disable Account"</button>
                    </div>
                </div>
                <div class="section">
                    <h2>"Submissions"</h2>
                </div>
            </Suspense>
        </section>
    }
}

#[component]
fn Username(action: ServerAction<Update>, show: ReadSignal<bool>) -> impl IntoView {
    let result = Signal::derive(move || action.value().get().unwrap_or(Ok(())));
    let (username, set_username) = signal(String::new());
    view! {
        <section id="box" class:hidden=move || !show.get()>
            <h1>"Edit Username"</h1>
            <ErrorBoundary fallback=|e| {
                view! {
                    <span class="error">
                        {move || {
                            let e = e.get().into_iter().next().unwrap().1;
                            if e.is::<ServerFnError<ApiError>>() {
                                let e = e.downcast_ref::<ServerFnError<ApiError>>().unwrap();
                                match e {
                                    ServerFnError::WrappedServerError(err) => {
                                        match err {
                                            ApiError::AlreadyExists => "🛈 Username already exists",
                                            _ => "🛈 Something went wrong. Try again",
                                        }
                                    }
                                    _ => "🛈 Something went wrong. Try again",
                                }
                            } else {
                                "🛈 Something went wrong. Try again"
                            }
                        }}
                    </span>
                }
            }>
                <div class="hidden">{result}</div>
            </ErrorBoundary>
            <ActionForm action>
                <div class="input-box">
                    <input
                        type="text"
                        name="username"
                        id="username"
                        required
                        minlength="2"
                        maxlength="32"
                        pattern="[a-zA-Z_\\-\\.]*"
                        value=username
                        on:input=move |e| set_username(event_target_value::<Event>(&e))
                    />
                    <label for="username" class="placeholder">
                        "Username"
                    </label>
                    <label for="username" class="error">
                        "Username must adhere to [a-zA-Z_-.]{2,32}."
                    </label>
                </div>
                <input type="submit" class="button" value="Save" />
            </ActionForm>
        </section>
    }
}

#[component]
fn Avatar(
    action: Action<FormData, Result<(), ServerFnError<ApiError>>, LocalStorage>,
    show: ReadSignal<bool>,
) -> impl IntoView {
    let result = Signal::derive(move || action.value().get().unwrap_or(Ok(())));
    let (username, set_username) = signal(String::new());
    view! {
        <section id="box" class:hidden=move || !show.get()>
            <h1>"Change Avatar"</h1>
            <ErrorBoundary fallback=|e| {
                view! {
                    <span class="error">
                        {move || {
                            let e = e.get().into_iter().next().unwrap().1;
                            if e.is::<ServerFnError<ApiError>>() {
                                let e = e.downcast_ref::<ServerFnError<ApiError>>().unwrap();
                                match e {
                                    ServerFnError::Args(_) => "🛈 File must be jpg and under 1 MB",
                                    _ => "🛈 Something went wrong. Try again",
                                }
                            } else {
                                "🛈 Something went wrong. Try again"
                            }
                        }}
                    </span>
                }
            }>
                <div class="hidden">{result}</div>
            </ErrorBoundary>
            <form on:submit=move |ev: SubmitEvent| {
                ev.prevent_default();
                let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
                let form_data = FormData::new_with_form(&target).unwrap();
                action.dispatch_local(form_data);
            }>
                <div class="input-box">
                    <input
                        type="text"
                        name="username"
                        id="username"
                        required
                        minlength="2"
                        maxlength="32"
                        pattern="[a-zA-Z_\\-\\.]*"
                        value=username
                        on:input=move |e| set_username(event_target_value::<Event>(&e))
                    />
                    <label for="username" class="placeholder">
                        "Username"
                    </label>
                    <label for="username" class="error">
                        "Username must adhere to [a-zA-Z_-.]{2,32}."
                    </label>
                </div>
                <input type="submit" class="button" value="Save" />
            </form>
        </section>
    }
}
