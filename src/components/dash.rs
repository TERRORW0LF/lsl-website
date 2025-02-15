use crate::{
    app::{UpdatePfpAction, UserResource},
    server::{
        api::ApiError,
        auth::{discord_list, DiscordAdd, DiscordDelete, Logout, Update},
    },
};
use leptos::{either::Either, html::Input, prelude::*};
use leptos_meta::Title;
use leptos_router::components::{Outlet, A};
use wasm_bindgen::JsCast;
use web_sys::{Event, FormData, HtmlFormElement, SubmitEvent};

#[component]
pub fn Dashboard() -> impl IntoView {
    let user = expect_context::<UserResource>();
    let logout = expect_context::<ServerAction<Logout>>();

    view! {
        <Title text="Dashboard" />
        <section id="dashboard">
            <Outlet />
            <Suspense fallback=|| view! { <h1>"Loading..."</h1> }>
                <h1>"Dashboard"</h1>
                <div class="section">
                    <h2>"Profile"</h2>
                    <div class="row">
                        <A href="avatar">
                            <img
                                class="pfp"
                                src=move || {
                                    format!(
                                        "/cdn/users/{}.jpg",
                                        user
                                            .get()
                                            .map(|res| { res.unwrap_or_default().pfp })
                                            .unwrap_or("default".into()),
                                    )
                                }
                            />
                        </A>
                        <A attr:class="button primary" href="avatar">
                            "Change Avatar"
                        </A>
                    </div>
                    <div class="spacer-2"></div>
                    <div class="row">
                        <div class="narrow">
                            <h3>"USERNAME"</h3>
                            <h4>
                                {move || { user.get().map(|user| user.map(|user| user.username)) }}
                            </h4>
                        </div>
                        <A attr:class="button secondary" href="username">
                            "Edit"
                        </A>
                    </div>
                    <div class="spacer-2"></div>
                    <div class="row">
                        <div class="narrow">
                            <h3>"PASSWORD"</h3>
                            <h4>"********"</h4>
                        </div>
                        <A attr:class="button secondary" href="password">
                            "Edit"
                        </A>
                    </div>
                </div>
                <div class="section">
                    <h2>"Account"</h2>
                    <h3>"CONNECTIONS"</h3>
                    <div>
                        <h4>"Discord"</h4>
                        <p>
                            "Manage your connected Discord profiles or add a new one. "
                            "This allows you to easily submit and manage your runs from within Discord."
                        </p>

                        <A attr:class="button primary" href="discord">
                            "Manage"
                        </A>
                    </div>
                    <h3>"MANAGEMENT"</h3>
                    <div>
                        <h4>"Log Out"</h4>
                        <p>"Logging out of your account will only affect this device."</p>
                        <ActionForm action=logout>
                            <input type="submit" class="button secondary" value="Log Out" />
                        </ActionForm>
                    </div>
                    <div>
                        <h4>"Account Removal"</h4>
                        <p>
                            "Disabling your account will anonymize your account and prevent "
                            "you from logging in. This action will not delete runs from the leaderboard."
                        </p>
                        <button class="button danger">"Disable Account"</button>
                    </div>
                </div>
                <div class="section">
                    <h2>"Submissions"</h2>
                    <div>
                        <h4>"Edit Submissions"</h4>
                        <p>
                            "Manage your submissions to the leaderboards. "
                            "This will redirect you to the submission management page."
                        </p>
                        <A attr:class="button primary" href="../manage">
                            "Edit"
                        </A>
                    </div>
                    <div>
                        <h4>"Claim Submissions"</h4>
                        <p>
                            "Claim submissions made under the old Google Sheets method "
                            "to your account. Requires moderator approval."
                        </p>
                        <button class="button secondary">"Claim"</button>
                    </div>
                </div>
            </Suspense>
        </section>
    }
}

#[component]
pub fn Username() -> impl IntoView {
    let action = expect_context::<ServerAction<Update>>();
    let result = Signal::derive(move || action.value().get());
    let (username, set_username) = signal(String::new());
    view! {
        <A attr:class="toner" href="../">
            <div />
        </A>
        <section id="box">
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
                <input type="text" name="redirect" hidden value="user/@me/dashboard" />
                <div class="input-box">
                    <input
                        type="text"
                        name="username"
                        id="username"
                        required
                        minlength="2"
                        maxlength="32"
                        pattern="[a-zA-Z0-9_\\-\\.]*"
                        value=username
                        on:input=move |e| set_username(event_target_value::<Event>(&e))
                    />
                    <label for="username" class="placeholder">
                        "Username"
                    </label>
                    <label for="username" class="error">
                        "Username must adhere to [a-zA-Z0-9_-.]{2,32}."
                    </label>
                </div>
                <div class="row">
                    <A attr:class="button secondary" href="../">
                        "Cancel"
                    </A>
                    <input type="submit" class="button primary" value="Save" />
                </div>
            </ActionForm>
        </section>
    }
}

#[component]
pub fn Password() -> impl IntoView {
    let action = expect_context::<ServerAction<Update>>();
    let result = Signal::derive(move || action.value().get());
    let (password, set_password) = signal(String::new());
    let (password_new, set_password_new) = signal(String::new());
    let (password_rep, set_password_rep) = signal(String::new());
    view! {
        <A attr:class="toner" href="../">
            <div />
        </A>
        <section id="box">
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
                <input type="text" name="redirect" hidden value="user/@me/dashboard" />
                <div class="input-box">
                    <input
                        type="password"
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
                        "New Password"
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
                        "Repeat Password"
                    </label>
                    <label for="password-repeat" class="error">
                        "Passwords must match."
                    </label>
                </div>
                <div class="row">
                    <A attr:class="button secondary" href="../">
                        "Cancel"
                    </A>
                    <input type="submit" class="button primary" value="Save" />
                </div>
            </ActionForm>
        </section>
    }
}

#[component]
pub fn Avatar() -> impl IntoView {
    let action = expect_context::<UpdatePfpAction>();
    let result = Signal::derive(move || action.value().get());
    let input_ref = NodeRef::<Input>::new();
    view! {
        <A attr:class="toner" href="../">
            <div />
        </A>
        <section id="box">
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
                    <input
                        node_ref=input_ref
                        type="file"
                        name="avatar"
                        id="avatar"
                        required
                        accept=".jpg"
                    />
                    <label
                        for="avatar"
                        class="file"
                        on:dragover=move |ev| ev.prevent_default()
                        on:drop=move |ev| {
                            ev.prevent_default();
                            if let Some(dt) = ev.data_transfer() {
                                if let Some(input) = input_ref.get_untracked() {
                                    input.set_files(dt.files().as_ref());
                                }
                            }
                        }
                    >
                        <h6>"Drag and drop file here or click to open file selector"</h6>
                        <p>"Accepts .jpg files under 4 MB"</p>
                    </label>
                    <label for="avatar" class="error">
                        "File must be jpg."
                    </label>
                </div>
                <div class="row">
                    <A attr:class="button secondary" href="../">
                        "Cancel"
                    </A>
                    <input type="submit" class="button primary" value="Save" />
                </div>
            </form>
        </section>
    }
}

#[component]
pub fn DiscordList() -> impl IntoView {
    let discord_del = ServerAction::<DiscordDelete>::new();
    let discord_add = ServerAction::<DiscordAdd>::new();
    let discord_list = Resource::new(
        move || (discord_del.version().get(), discord_add.version().get()),
        |_| discord_list(),
    );
    view! {
        <A attr:class="toner" href="../">
            <div />
        </A>
        <section id="box">
            <h1>"Discord"</h1>
            <Transition fallback=move || {
                view! { <p>"Loading..."</p> }
            }>
                <ErrorBoundary fallback=|_| {
                    view! { <span class="error">"🛈 Something went wrong. Try again"</span> }
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
                                                        <div class="narrow">
                                                            <h4>{con.name}</h4>
                                                            <p>{con.snowflake.clone()}</p>
                                                        </div>
                                                        <ActionForm action=discord_del>
                                                            <input
                                                                hidden
                                                                type="text"
                                                                name="snowflake"
                                                                value=con.snowflake.clone()
                                                            />
                                                            <input type="submit" class="button danger" value="Remove" />
                                                        </ActionForm>
                                                    </div>
                                                }
                                            })
                                            .collect_view()}
                                        {if conns.len() < 5 {
                                            Either::Left(
                                                view! {
                                                    <ActionForm action=discord_add>
                                                        <input
                                                            type="submit"
                                                            class="discord-add"
                                                            value="Add Account"
                                                        />
                                                    </ActionForm>
                                                },
                                            )
                                        } else {
                                            Either::Right(())
                                        }}
                                    }
                                })
                            })
                    }}
                </ErrorBoundary>
            </Transition>
            <div class="row">
                <A attr:class="button secondary" href="../">
                    "Cancel"
                </A>
                <A attr:class="button primary" href="../">
                    "Save"
                </A>
            </div>
        </section>
    }
}
