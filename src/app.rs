use crate::{
    components::{
        auth::{Login, Register, Submit},
        dash::Dashboard,
        error_template::{AppError, ErrorTemplate},
        home::HomePage,
        leaderboard::{Leaderboard, Section},
    },
    server::{
        api::ApiError,
        auth::{get_user, pfp, Login, Logout, Register, Update, User},
    },
};
use leptos::{either::*, prelude::*};
use leptos_meta::MetaTags;
use leptos_meta::*;
use leptos_router::{
    components::{ParentRoute, ProtectedRoute, Redirect, Route, Router, Routes, A},
    path, MatchNestedRoutes,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::FormData;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" dir="ltr">
            <head>
                <Link rel="preconnect" href="https://fonts.googleapis.com" />
                <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="" />
                <Link
                    href="https://fonts.googleapis.com/css2?family=Roboto+Flex:opsz,wght@8..144,100..1000&display=swap"
                    rel="stylesheet"
                />
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let login = ServerAction::<Login>::new();
    let logout = ServerAction::<Logout>::new();
    let register = ServerAction::<Register>::new();
    let update = ServerAction::<Update>::new();
    let update_pfp = Action::new_local(|data: &FormData| pfp(data.clone().into()));
    let user = Resource::new(
        move || {
            (
                login.version().get(),
                register.version().get(),
                logout.version().get(),
                update.version().get(),
                update_pfp.version().get(),
            )
        },
        move |_| get_user(),
    );

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    Effect::new(|_| {
        document()
            .document_element()
            .unwrap()
            .set_class_name("dark")
    });

    Effect::new(|_| {
        let a = Closure::<dyn FnMut()>::new(|| {
            let _ = document()
                .body()
                .unwrap()
                .style()
                .set_property("--scroll", &(window().page_y_offset().unwrap()).to_string());
        });
        let _ = window().add_event_listener_with_callback_and_bool(
            "scroll",
            a.as_ref().unchecked_ref(),
            false,
        );
        a.forget();
    });

    view! {
        <Stylesheet id="leptos" href="/pkg/lsl-website.css" />
        // sets the document title
        <Title text="Lucio Surf League" />
        // content for this welcome page
        <Router>
            <header>
                <nav class="split-row-nav">
                    <div class="left-row-nav">
                        <A href="/home">"Home"</A>
                        <A href="/leaderboard/2.00/1/standard">"Leaderboard"</A>
                        <A href="/ranking">"Ranking"</A>
                        <a href="https://discord.com/invite/G9QBCDY" rel="external">
                            "Discord"
                        </a>
                    </div>
                    <div class="right-row-nav">
                        <Transition fallback=move || {
                            view! { <span>"Loading..."</span> }
                        }>
                            {move || {
                                user.get()
                                    .map(|user| match user {
                                        Err(e) => {
                                            EitherOf3::A(
                                                view! {
                                                    <A href="/login">"Login"</A>
                                                    <span>{format!("Login error: {}", e)}</span>
                                                },
                                            )
                                        }
                                        Ok(None) => {
                                            EitherOf3::B(view! { <A href="/login">"Login"</A> })
                                        }
                                        Ok(Some(user)) => {
                                            EitherOf3::C(
                                                view! {
                                                    <A href="/dashboard">
                                                        <img src=format!("/cdn/users/{}.jpg", user.pfp) />
                                                    </A>
                                                },
                                            )
                                        }
                                    })
                            }}
                        </Transition>
                    </div>
                </nav>
            </header>
            <main>
                <AppRouter register login update update_pfp logout user />
            </main>
        </Router>
    }
}

#[component(transparent)]
fn AppRouter(
    register: ServerAction<Register>,
    login: ServerAction<Login>,
    update: ServerAction<Update>,
    update_pfp: Action<FormData, Result<(), ServerFnError<ApiError>>, LocalStorage>,
    logout: ServerAction<Logout>,
    user: Resource<Result<Option<User>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <Routes fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors /> }
        }>
            <Route path=path!("") view=|| view! { <Redirect path="home" /> } />
            <Route path=path!("home") view=HomePage />
            <Route
                path=path!("discord")
                view=|| {
                    Effect::new(|_| {
                        window().location().set_href("https://discord.com/invite/G9QBCDY")
                    });
                    ""
                }
            />
            <Route path=path!("register") view=move || view! { <Register action=register /> } />
            <Route path=path!("login") view=move || view! { <Login action=login /> } />
            <ProtectedRoute
                path=path!("dashboard")
                condition=move || user.get().map(|n| n.map_or(false, |u| u.is_some()))
                redirect_path=|| "/login?redirect=dashboard"
                view=move || view! { <Dashboard user update update_pfp logout /> }
            />
            <ProtectedRoute
                path=path!("submit")
                condition=move || user.get().map(|n| n.map_or(false, |u| u.is_some()))
                redirect_path=|| "/login?redirect=submit"
                view=Submit
            />
            <LeaderboardRouter />
        </Routes>
    }
}

#[component(transparent)]
fn LeaderboardRouter() -> impl MatchNestedRoutes + Clone {
    let (patch, set_patch) = signal(String::new());
    let (layout, set_layout) = signal(String::new());
    let (category, set_category) = signal(String::new());

    view! {
        <ParentRoute
            path=path!("leaderboard")
            view=move || view! { <Section patch=patch layout=layout category=category /> }
        >
            <Route
                path=path!(":patch/:layout/:category")
                view=move || {
                    view! {
                        <Leaderboard patch=set_patch layout=set_layout category=set_category />
                    }
                }
            />
            <Route path=path!("") view=|| view! { <Redirect path="2.00/1/standard" /> } />
            <Route
                path=path!(":patch")
                view=|| {
                    view! { <Redirect path="1/standard" /> }
                }
            />
            <Route
                path=path!(":patch/:layout")
                view=|| {
                    view! { <Redirect path="standard" /> }
                }
            />
        </ParentRoute>
    }
    .into_inner()
}
