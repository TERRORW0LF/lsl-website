use leptos::{either::*, prelude::*};
use leptos_meta::Title;
use leptos_router::{components::A, hooks::use_query_map};
use server::{
    api::get_maps,
    auth::{Login, Register, Submit},
};
use types::api::{ApiError, Map};
use web_sys::Event;

#[component]
pub fn Login() -> impl IntoView {
    let action = expect_context::<ServerAction<Login>>();
    let result = Signal::derive(move || action.value().get().unwrap_or(Ok(())));
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    view! {
        <Title text="Log In" />
        <section id="box">
            <h1>"Sign In"</h1>
            <ErrorBoundary fallback=|e| {
                view! {
                    <span class="error">
                        {move || {
                            let e = e.get().into_iter().next().unwrap().1;
                            if e.is::<ApiError>() {
                                let e = e.downcast_ref::<ApiError>().unwrap();
                                match e {
                                    ApiError::InvalidCredentials => {
                                        "🛈 Incorrect username or password"
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
            <ActionForm action=action>
                <input
                    hidden
                    type="text"
                    name="redirect"
                    value=use_query_map().get().get("redirect")
                    prop:value=use_query_map().get().get("redirect")
                />
                <div class="input-box">
                    <input
                        type="text"
                        name="username"
                        id="username"
                        required
                        value=username
                        on:input=move |e| set_username(event_target_value::<Event>(&e))
                    />
                    <label for="username" class="placeholder">
                        "Username"
                    </label>
                </div>
                <div class="input-box">
                    <input
                        type="password"
                        name="password"
                        id="password"
                        required
                        value=password
                        on:input=move |e| set_password(event_target_value::<Event>(&e))
                    />
                    <label for="password" class="placeholder">
                        "Password"
                    </label>
                </div>
                <div class="remember">
                    <input type="checkbox" name="remember" id="remember" />
                    <label for="remember">"Remember me?"</label>
                </div>
                <div class="row">
                    <A href="/register" attr:class="link">
                        "Create account"
                    </A>
                    <input type="submit" class="button primary" value="Sign In" />
                </div>
            </ActionForm>
        </section>
    }
}

#[component]
pub fn Register() -> impl IntoView {
    let action = expect_context::<ServerAction<Register>>();
    let result = Signal::derive(move || action.value().get().unwrap_or(Ok(())));
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (password_rep, set_password_rep) = signal(String::new());
    view! {
        <Title text="Sign Up" />
        <section id="box">
            <h1>"Sign Up"</h1>
            <ErrorBoundary fallback=|e| {
                view! {
                    <span class="error">
                        {move || {
                            let e = e.get().into_iter().next().unwrap().1;
                            if e.is::<ApiError>() {
                                let e = e.downcast_ref::<ApiError>().unwrap();
                                match e {
                                    ApiError::AlreadyExists => "🛈 Username already exists",
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
            <ActionForm action=action>
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
                <div class="input-box">
                    <input
                        type="password"
                        name="password"
                        id="password"
                        required
                        minlength="8"
                        maxlength="256"
                        value=password
                        on:input=move |e| set_password(event_target_value::<Event>(&e))
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
                        name="password_confirm"
                        id="password-repeat"
                        required
                        pattern=password
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
                <div class="remember">
                    <input type="checkbox" name="remember" id="remember" />
                    <label for="remember">"Remember me?"</label>
                </div>
                <input type="submit" class="button primary" value="Sign Up" />
            </ActionForm>
        </section>
    }
}

#[component]
pub fn Submit() -> impl IntoView {
    let action = ServerAction::<Submit>::new();
    let result = Signal::derive(move || action.value().get().unwrap_or(Ok(())));
    let (code, set_code) = signal(String::new());
    let (time, set_time) = signal(String::new());
    let (map, set_map) = signal(String::new());
    let (yt, set_yt) = signal(String::new());
    let (maps, set_maps) = signal::<Option<Vec<Map>>>(None);
    view! {
        <Title text="Submit" />
        <section id="box">
            <h1>"Submit"</h1>
            <ErrorBoundary fallback=|e| {
                view! {
                    <span class="error">
                        {move || {
                            let e = e.get().into_iter().next().unwrap().1;
                            if e.is::<ApiError>() {
                                let e = e.downcast_ref::<ApiError>().unwrap();
                                match e {
                                    ApiError::InvalidSection => "🛈 Invalid map name",
                                    ApiError::InvalidYtId => "🛈 YT video id does not exist",
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
            <ActionForm action=action>
                <div class="row no-wrap">
                    <div class="input-box">
                        <input
                            type="text"
                            id="code"
                            minlength="4"
                            maxlength="4"
                            pattern="[1-5][sgSG][a-zA-Z0-9]{2}"
                            value=code
                            on:input=move |e| {
                                let c = event_target_value::<Event>(&e);
                                match maps.get() {
                                    Some(ms) => {
                                        let id: String = c
                                            .chars()
                                            .into_iter()
                                            .skip(2)
                                            .take(2)
                                            .collect::<String>()
                                            .to_ascii_uppercase();
                                        let m = ms.into_iter().find(|m| m.code[2..4] == id);
                                        match m {
                                            Some(m) => set_map(m.name),
                                            None => set_map(String::new()),
                                        };
                                    }
                                    None => {}
                                };
                                set_code(c)
                            }
                        />
                        <label for="code" class="placeholder">
                            "Code ∗"
                        </label>
                        <label for="code" class="error">
                            "Invalid code."
                        </label>
                    </div>
                    <div class="input-box">
                        <input
                            required
                            type="number"
                            name="time"
                            id="time"
                            min="0"
                            step="0.001"
                            value=time
                            on:input=move |e| set_time(event_target_value::<Event>(&e))
                        />
                        <label for="time" class="placeholder">
                            "Time"
                        </label>
                        <label for="time" class="error">
                            "Invalid time."
                        </label>
                    </div>
                </div>
                <div class="row no-wrap">
                    <div class="input-box">
                        <select
                            required
                            name="layout"
                            id="layout"
                            prop:selectedIndex=move || {
                                code.get()
                                    .chars()
                                    .next()
                                    .map_or(0, |c| c.to_digit(10).unwrap_or(0) as i32 - 1)
                            }
                        >
                            <option value="1">"Layout 1"</option>
                            <option value="2">"Layout 2"</option>
                            <option value="3">"Layout 3"</option>
                            <option value="4">"Layout 4"</option>
                            <option value="5">"Layout 5"</option>
                        </select>
                        <label for="layout" class="indicator">
                            "Layout"
                        </label>
                    </div>
                    <div class="input-box">
                        <select
                            required
                            name="category"
                            id="category"
                            prop:selectedIndex=move || {
                                code.get()
                                    .chars()
                                    .nth(1)
                                    .map_or(
                                        0,
                                        |c| match c.to_ascii_lowercase() {
                                            's' => 0,
                                            'g' => 1,
                                            _ => -1,
                                        },
                                    )
                            }
                        >
                            <option value="Standard">"Standard"</option>
                            <option value="Gravspeed">"Gravspeed"</option>
                        </select>
                        <label for="category" class="indicator">
                            "Category"
                        </label>
                    </div>
                </div>
                <div class="input-box">
                    <input
                        list="maps"
                        name="map"
                        id="map"
                        value=map
                        prop:value=map
                        on:input=move |e| set_map(event_target_value::<Event>(&e))
                    />
                    <datalist id="maps">
                        <Await future=get_maps() let:maps>
                            {match maps {
                                Ok(v) => {
                                    set_maps(Some(v.clone()));
                                    Either::Left(
                                        v
                                            .into_iter()
                                            .map(|m| {
                                                view! {
                                                    <option value=m.name.clone()>{m.name.clone()}</option>
                                                }
                                            })
                                            .collect_view(),
                                    )
                                }
                                Err(_) => Either::Right(view! {}),
                            }}
                        </Await>
                    </datalist>
                    <label for="map" class="placeholder">
                        "Map"
                    </label>
                </div>
                <div class="input-box">
                    <input
                        type="text"
                        name="yt_id"
                        id="yt_id"
                        required
                        minlength="11"
                        maxlength="11"
                        pattern="[a-zA-Z0-9_\\-]*"
                        value=yt
                        on:input=move |e| set_yt(event_target_value::<Event>(&e))
                    />
                    <label for="yt_id" class="placeholder">
                        "Video ID"
                    </label>
                    <label for="yt_id" class="error">
                        "Invalid video ID."
                    </label>
                </div>
                <div class="text">"∗ Fields are optional."</div>
                <input type="submit" class="button primary" value="Submit" />
            </ActionForm>
        </section>
    }
}
