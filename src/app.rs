use crate::{
    auth::{get_user, Login, Logout, Register},
    error_template::{AppError, ErrorTemplate},
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    let login = create_server_action::<Login>();
    let logout = create_server_action::<Logout>();
    let register = create_server_action::<Register>();
    let user = create_resource(
        move || {
            (
                login.version().get(),
                register.version().get(),
                logout.version().get(),
            )
        },
        move |_| get_user(),
    );

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/lsl-website.css" />

        // sets the document title
        <Title text="Welcome to Leptos" />

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors /> }.into_view()
        }>
            <header>
                <nav id="site-nav">
                    <div class="site-nav">
                        <A href="/">"Lucio Surf League"</A>
                        <A href="/leaderboard">"Leaderboard"</A>
                    </div>
                    <div class="user-nav">
                        <Transition fallback=move || {
                            view! { <span>"Loading..."</span> }
                        }>
                            {move || {
                                user.get()
                                    .map(|user| match user {
                                        Err(e) => {
                                            view! {
                                                <A href="/register">"Register"</A>
                                                <A href="/login">"Login"</A>
                                                <span>{format!("Login error: {}", e)}</span>
                                            }
                                                .into_view()
                                        }
                                        Ok(None) => {
                                            view! {
                                                <A href="/register">"Register"</A>
                                                <A href="/login">"Login"</A>
                                                <span>"Logged out."</span>
                                            }
                                                .into_view()
                                        }
                                        Ok(Some(user)) => {
                                            view! {
                                                <A href="/settings">"Settings"</A>
                                                <span>
                                                    {format!("Logged in as: {} ({})", user.username, user.id)}
                                                </span>
                                            }
                                                .into_view()
                                        }
                                    })
                            }}
                        </Transition>
                    </div>
                </nav>
            </header>
            <main>
                <Routes>
                    <Route path="" view=HomePage />
                    <Route path="register" view=move || view! { <Register action=register /> } />
                    <Route path="login" view=move || view! { <Login action=login /> } />
                    <Route
                        path="settings"
                        view=move || {
                            view! {
                                <h1>"Settings"</h1>
                                <Logout action=logout />
                            }
                        }
                    />
                    <Route path="leaderboard" view=Section>
                        <Route path=":patch" view=Leaderboard />
                        <Route path="" view=|| view! { "Please select a patch." } />
                    </Route>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <section id="welcome">
            <h1>"Welcome to the Lucio Surf League!"</h1>
            <p>
                "Since its inception in 2019 "<b>"Lucio Surf"</b>
                " has been a staple in Overwatch's custom game browser. The "
                <b>"Lucio Surf League"</b>
                " has been the proud maintainer and arbitrator of Lucio Surf since the beginning. "
                "Updating the game mode and leaderboards, connecting the community, and holding tournaments, the Lucio Surf League "
                "is your place for anything related to Lucio Surf. We welcome you as another frog in our ranks."
            </p>
        </section>
        <section id="potd">
            <h2>"Player of the day"</h2>
            <div>
                <img src="" />
                <p>
                    "Test is an exceptional player combining superb routing with steady movement. This allows them to top "
                    "any leaderboard that don't involve boosts. While occasionaly dabbling in Gravspeed their main category "
                    "is Standard, prefering the challenge over the speed."
                </p>
            </div>
        </section>
    }
}

#[component]
pub fn Section() -> impl IntoView {
    let (patch, set_patch) = create_signal(-1);
    let on_patch = move |n: i32| {
        patch.with(|patch| {
            if *patch == n {
                "selected".to_string()
            } else {
                String::new()
            }
        })
    };

    view! {
        <section id="section">
            <nav id="leaderboard-nav">
                <ul>
                    <li class=move || on_patch(0)>
                        <A on:click=move |_| set_patch(0) href="1.00">
                            "1.00"
                        </A>
                    </li>
                    <li class=move || on_patch(1)>
                        <A on:click=move |_| set_patch(1) href="1.41">
                            "1.41"
                        </A>
                    </li>
                    <li class=move || on_patch(2)>
                        <A on:click=move |_| set_patch(2) href="1.50">
                            "1.50"
                        </A>
                    </li>
                    <li class=move || on_patch(3)>
                        <A on:click=move |_| set_patch(3) href="2.00">
                            "current"
                        </A>
                    </li>
                </ul>
            </nav>
            <Outlet />
        </section>
    }
}

#[component]
pub fn Leaderboard() -> impl IntoView {
    "Loading"
}

#[component]
pub fn Login(action: Action<Login, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <ActionForm action=action>
            <h1>"Log In"</h1>
            <label>
                "User ID:"
                <input
                    type="text"
                    placeholder="User ID"
                    maxlength="32"
                    name="username"
                    class="auth-input"
                />
            </label>
            <br />
            <label>
                "Password:"
                <input type="password" placeholder="Password" name="password" class="auth-input" />
            </label>
            <br />
            <label>
                <input type="checkbox" name="remember" class="auth-input" />
                "Remember me?"
            </label>
            <br />
            <button type="submit" class="button">
                "Log In"
            </button>
        </ActionForm>
    }
}

#[component]
pub fn Register(action: Action<Register, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <ActionForm action=action>
            <h1>"Sign Up"</h1>
            <label>
                "User ID:"
                <input
                    type="text"
                    placeholder="User ID"
                    maxlength="32"
                    name="username"
                    class="auth-input"
                />
            </label>
            <br />
            <label>
                "Password:"
                <input type="password" placeholder="Password" name="password" class="auth-input" />
            </label>
            <br />
            <label>
                "Confirm Password:"
                <input
                    type="password"
                    placeholder="Password again"
                    name="password_confirm"
                    class="auth-input"
                />
            </label>
            <br />
            <label>
                "Remember me?" <input type="checkbox" name="remember" class="auth-input" />
            </label>

            <br />
            <button type="submit" class="button">
                "Sign Up"
            </button>
        </ActionForm>
    }
}

#[component]
pub fn Logout(action: Action<Logout, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <div id="loginbox">
            <ActionForm action=action>
                <button type="submit" class="button">
                    "Log Out"
                </button>
            </ActionForm>
        </div>
    }
}
