use crate::server::{
    api::ApiError,
    auth::{discord_delete, discord_list, Discord, Logout, Update, User},
};
use leptos::{either::Either, prelude::*};
use leptos_meta::Title;
use wasm_bindgen::JsCast;
use web_sys::{Event, FormData, HtmlFormElement, SubmitEvent};

#[derive(PartialEq, Eq)]
enum PopUp {
    None,
    Username,
    Password,
    Avatar,
    Discord,
}

#[component]
pub fn Dashboard(
    user: Resource<Result<Option<User>, ServerFnError>>,
    update: ServerAction<Update>,
    update_pfp: Action<FormData, Result<(), ServerFnError<ApiError>>, LocalStorage>,
    logout: ServerAction<Logout>,
) -> impl IntoView {
    let show = RwSignal::new(PopUp::None);
    let discord_del = Action::new(|snowflake: &String| {
        let snowflake = snowflake.to_owned();
        async move { discord_delete(snowflake).await }
    });
    let discord_add = Action::new(|input: &String| {
        let input = input.to_owned();
        async move { discord_delete(input).await }
    });
    let discord_list = Resource::new(
        move || (discord_del.version().get(), discord_add.version().get()),
        |_| discord_list(),
    );
    view! {
        <Title text="Dashboard" />
        <div
            class="toner"
            class:hidden=move || { !(*show.read() == PopUp::None) }
            on:click=move |_| { show.set(PopUp::None) }
        />
        <Username action=update show />
        <Password action=update show />
        <Avatar action=update_pfp show />
        <DiscordList discord_list discord_del show />
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
                            on:click=move |_| show.set(PopUp::Avatar)
                        />
                        <button class="primary" on:click=move |_| show.set(PopUp::Avatar)>
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
                        <button class="secondary" on:click=move |_| show.set(PopUp::Username)>
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
                </div>
                <div class="section">
                    <h2>"Account"</h2>
                    <h3>"CONNECTIONS"</h3>
                    <div>
                        <h4>"Discord"</h4>
                        <p>
                            "Manage your connected Discord profiles or add a new one."
                            "This allows you to easily submit and manage your runs from within Discord."
                        </p>
                        <button class="primary">"Manage"</button>
                    </div>
                    <h3>"MANAGEMENT"</h3>
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
                            "Disabling your account will anonymize your account and prevent"
                            "you from logging in. This action will not delete runs from the leaderboard."
                        </p>
                        <button class="secondary">"Disable Account"</button>
                    </div>
                </div>
                <div class="section">
                    <h2>"Submissions"</h2>
                    <div>
                        <h4>"Edit Submissions"</h4>
                        <p>
                            "Manage your submissions to the leaderboards."
                            "This will redirect you to the submission management page."
                        </p>
                        <button class="primary">"Edit"</button>
                    </div>
                    <div>
                        <h4>"Claim Submissions"</h4>
                        <p>
                            "Claim submissions made under the old Google Sheets method"
                            "to your account. Requires moderator approval."
                        </p>
                        <button class="primary">"Claim"</button>
                    </div>
                </div>
            </Suspense>
        </section>
    }
}

#[component]
fn Username(action: ServerAction<Update>, show: RwSignal<PopUp>) -> impl IntoView {
    let result = Signal::derive(move || action.value().get().unwrap_or(Ok(())));
    let (username, set_username) = signal(String::new());
    view! {
        <section id="box" class:hidden=move || !(*show.read() == PopUp::Username)>
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
                <div class="row">
                    <button
                        class="secondary"
                        on:click=move |_| {
                            show.set(PopUp::None);
                        }
                    >
                        "Cancel"
                    </button>
                    <input type="submit" class="button" value="Save" />
                </div>
            </ActionForm>
        </section>
    }
}

#[component]
fn Password(action: ServerAction<Update>, show: RwSignal<PopUp>) -> impl IntoView {
    let result = Signal::derive(move || action.value().get().unwrap_or(Ok(())));
    let (password, set_password) = signal(String::new());
    let (password_new, set_password_new) = signal(String::new());
    let (password_rep, set_password_rep) = signal(String::new());
    view! {
        <section id="box" class:hidden=move || !(*show.read() == PopUp::Password)>
            <h1>"Edit Password"</h1>
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
                                            ApiError::InvalidCredentials => "🛈 Incorrect password",
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
                        name="password[old]"
                        id="password"
                        required
                        value=password
                        on:input=move |e| set_password(event_target_value::<Event>(&e))
                    />
                    <label for="password" class="placeholder">
                        "Current Password"
                    </label>
                </div>
                <div class="input-box">
                    <input
                        type="password"
                        name="password[new]"
                        id="password"
                        required
                        minlength="8"
                        maxlength="256"
                        value=password_new
                        on:input=move |e| set_password_new(event_target_value::<Event>(&e))
                    />
                    <label for="password" class="placeholder">
                        "Password"
                    </label>
                    <label for="password" class="error">
                        "Password must be between 8 and 256 characters long."
                    </label>
                </div>
                <div class="input-box">
                    <input
                        type="password"
                        id="password-repeat"
                        required
                        pattern=password_new
                        value=password_rep
                        on:input=move |e| set_password_rep(event_target_value::<Event>(&e))
                    />
                    <label for="password-repeat" class="placeholder">
                        "Repeat password"
                    </label>
                    <label for="password-repeat" class="error">
                        "Passwords must match."
                    </label>
                </div>
                <div class="row">
                    <button
                        class="secondary"
                        on:click=move |_| {
                            show.set(PopUp::None);
                        }
                    >
                        "Cancel"
                    </button>
                    <input type="submit" class="button" value="Save" />
                </div>
            </ActionForm>
        </section>
    }
}

#[component]
fn Avatar(
    action: Action<FormData, Result<(), ServerFnError<ApiError>>, LocalStorage>,
    show: RwSignal<PopUp>,
) -> impl IntoView {
    let result = Signal::derive(move || action.value().get().unwrap_or(Ok(())));
    view! {
        <section id="box" class:hidden=move || !(*show.read() == PopUp::Avatar)>
            <h1>"Change Avatar"</h1>
            <ErrorBoundary fallback=|e| {
                view! {
                    <span class="error">
                        {move || {
                            let e = e.get().into_iter().next().unwrap().1;
                            if e.is::<ServerFnError<ApiError>>() {
                                let e = e.downcast_ref::<ServerFnError<ApiError>>().unwrap();
                                match e {
                                    ServerFnError::Args(_) => "🛈 File must be jpg and under 4 MB",
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
                    <input type="file" name="avatar" id="avatar" required accept=".jpg" />
                    <label for="avatar" class="file">
                        <h6>"Drag and drop file here or click to open file selector"</h6>
                        <p>"Accepts .jpg files under 4 MB"</p>
                    </label>
                    <label for="avatar" class="error">
                        "File must be jpg."
                    </label>
                </div>
                <div class="row">
                    <button
                        class="secondary"
                        on:click=move |_| {
                            show.set(PopUp::None);
                        }
                    >
                        "Cancel"
                    </button>
                    <input type="submit" class="button" value="Save" />
                </div>
            </form>
        </section>
    }
}

#[component]
fn DiscordList(
    discord_list: Resource<Result<Vec<Discord>, ServerFnError<ApiError>>>,
    discord_del: Action<String, Result<(), ServerFnError<ApiError>>>,
    show: RwSignal<PopUp>,
) -> impl IntoView {
    view! {
        <section id="box" class:hidden=move || !(*show.read() == PopUp::Discord)>
            <h1>"Select Account"</h1>
            <ErrorBoundary fallback=|_| {
                view! { <span class="error">"🛈 Something went wrong. Try again"</span> }
            }>
                <Transition fallback=move || {
                    view! { <p>"Loading..."</p> }
                }>
                    {move || {
                        discord_list
                            .get()
                            .map(|data| {
                                data.map(|conns| {
                                    view! {
                                        {conns
                                            .clone()
                                            .into_iter()
                                            .map(|con| {
                                                view! {
                                                    <div class="discord row">
                                                        <div>
                                                            <h3>{con.name}</h3>
                                                            <p>{con.snowflake.clone()}</p>
                                                        </div>
                                                        <button
                                                            class="danger"
                                                            on:click=move |_| {
                                                                discord_del.dispatch(con.snowflake.clone());
                                                            }
                                                        >
                                                            "Remove"
                                                        </button>
                                                    </div>
                                                }
                                            })
                                            .collect_view()}
                                        {if conns.len() < 5 {
                                            Either::Left(
                                                view! {
                                                    <button class="discord-add row">"Add Account"</button>
                                                },
                                            )
                                        } else {
                                            Either::Right(())
                                        }}
                                    }
                                })
                            })
                    }}
                </Transition>
            </ErrorBoundary>
        </section>
    }
}
