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
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/lsl-website.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <header>
                <A href="/">
                    <h1>"Lucio Surf League"</h1>
                </A>
                <Transition fallback = move || {
                    view! { <span>"Loading..."</span> }
                }>
                { move || {
                    user.get().map(|user| match user {
                        Err(e) => {
                            view! {
                                <A href="/register">"Register"</A>
                                ", "
                                <A href="/login">"Login"</A>
                                ", "
                                <span>{format!("Login error: {}", e)}</span>
                            }.into_view()
                        }
                        Ok(None) => {
                            view! {
                                <A href="/register">"Register"</A>
                                ", "
                                <A href="/login">"Login"</A>
                                ", "
                                <span>"Logged out."</span>
                            }.into_view()
                        }
                        Ok(Some(user)) => {
                            view! {
                                <A href="/settings">"Settings"</A>
                                ", "
                                <span>
                                    {format!("Logged in as: {} ({})", user.username, user.id)}
                                </span>
                            }
                                .into_view()
                        }
                    })
                }}
                </Transition>
            </header>
            <hr/>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="register" view=move || view! { <Register action=register/> }/>
                    <Route path="login" view=move || view! { <Login action=login/> }/>
                    <Route path="settings" view=move || {
                        view! {
                            <h1>"Settings"</h1>
                            <Logout action=logout/>
                        }
                    }/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
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
            <br/>
            <label>
                "Password:"
                <input type="password" placeholder="Password" name="password" class="auth-input"/>
            </label>
            <br/>
            <label>
                <input type="checkbox" name="remember" class="auth-input"/>
                "Remember me?"
            </label>
            <br/>
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
            <br/>
            <label>
                "Password:"
                <input type="password" placeholder="Password" name="password" class="auth-input"/>
            </label>
            <br/>
            <label>
                "Confirm Password:"
                <input
                    type="password"
                    placeholder="Password again"
                    name="password_confirm"
                    class="auth-input"
                />
            </label>
            <br/>
            <label>
                "Remember me?" <input type="checkbox" name="remember" class="auth-input"/>
            </label>

            <br/>
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
