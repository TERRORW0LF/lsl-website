use crate::{
    components::{
        auth::{Login, Register, Submit},
        dash::{Avatar, Bio, Dashboard, DiscordList, Password, Username},
        error_template::{AppError, ErrorTemplate},
        faq::FAQ,
        home::HomePage,
        leaderboard::{Leaderboard, Section},
        map::Map,
        ranking::Ranking,
        user::{Delete, ManageRuns, Profile},
    },
    server::{
        api::ApiError,
        auth::{
            get_current_user, update_pfp, Login, Logout, Register, UpdateBio, UpdateCreds, User,
        },
    },
};
use leptos::{either::*, prelude::*};
use leptos_meta::MetaTags;
use leptos_meta::*;
use leptos_router::{
    components::{
        ParentRoute, ProtectedParentRoute, ProtectedRoute, Redirect, Route, Router, Routes, A,
    },
    path, MatchNestedRoutes, NavigateOptions,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::FormData;

pub type UserResource = Resource<Result<User, ServerFnError<ApiError>>>;
pub type UpdatePfpAction = Action<FormData, Result<(), ServerFnError<ApiError>>, LocalStorage>;

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
                <Script src="https://cdn.jsdelivr.net/npm/echarts@5.5.1/dist/echarts.min.js" />
                <Script src="https://cdn.jsdelivr.net/npm/echarts-gl@2.0.9/dist/echarts-gl.min.js" />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body tabindex="-1">
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
    let update = ServerAction::<UpdateCreds>::new();
    let update_bio = ServerAction::<UpdateBio>::new();
    let update_pfp = Action::new_local(|data: &FormData| update_pfp(data.clone().into()));
    let user = Resource::new(
        move || {
            (
                login.version().get(),
                register.version().get(),
                logout.version().get(),
                update.version().get(),
                update_bio.version().get(),
                update_pfp.version().get(),
            )
        },
        move |_| get_current_user(),
    );

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    provide_context(user);
    provide_context(register);
    provide_context(login);
    provide_context(logout);
    provide_context(update);
    provide_context(update_bio);
    provide_context(update_pfp);

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
                    <ul class="left-row-nav">
                        <li>
                            <A href="/home">"Home"</A>
                        </li>
                        <li>
                            <A href="/leaderboard">"Leaderboard"</A>
                        </li>
                        <li>
                            <A href="/ranking">"Ranking"</A>
                        </li>
                        <li>
                            <a href="https://discord.com/invite/G9QBCDY" rel="external">
                                "Discord"
                            </a>
                        </li>
                    </ul>
                    <ul class="right-row-nav">
                        <Transition fallback=move || {
                            view! {
                                <li>
                                    <span>"Loading..."</span>
                                </li>
                            }
                        }>
                            {move || {
                                user.get()
                                    .map(|user| match user {
                                        Err(_) => {
                                            Either::Left(
                                                view! {
                                                    <li>
                                                        <A href="/login">"Login"</A>
                                                    </li>
                                                },
                                            )
                                        }
                                        Ok(user) => {
                                            Either::Right(
                                                view! {
                                                    <li>
                                                        <A href=format!("/user/{}/leaderboard", user.id)>
                                                            <img src=format!("/cdn/users/{}.jpg", user.pfp) />
                                                        </A>
                                                    </li>
                                                    <li class="dropdown">
                                                        <button
                                                            type="button"
                                                            class="dropdown-title"
                                                            aria-controls="user-dropdown"
                                                        >
                                                            "▼"
                                                        </button>
                                                        <ul class="dropdown-menu" id="user-dropdown">
                                                            <li>
                                                                <A href=format!(
                                                                    "/user/{}/leaderboard",
                                                                    user.id,
                                                                )>"Profile"</A>
                                                            </li>
                                                            <li>
                                                                <A href="/user/@me/submit">"Submit"</A>
                                                            </li>
                                                            <li>
                                                                <A href="/user/@me/dashboard">"Dashboard"</A>
                                                            </li>
                                                            <li>
                                                                <A href="/user/@me/manage">"Manage Runs"</A>
                                                            </li>
                                                            <li>
                                                                <button
                                                                    type="button"
                                                                    class="dropdown-title"
                                                                    on:click=move |_| {
                                                                        let _ = logout.dispatch(Logout {});
                                                                    }
                                                                >
                                                                    "Log Out"
                                                                </button>
                                                            </li>
                                                        </ul>
                                                    </li>
                                                },
                                            )
                                        }
                                    })
                            }}
                        </Transition>
                    </ul>
                </nav>
            </header>
            <main>
                <AppRouter />
            </main>
        </Router>
    }
}

#[component(transparent)]
fn AppRouter() -> impl IntoView {
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
            <Route path=path!("faq") view=FAQ />
            <Route path=path!("ranking") view=Ranking />
            <LeaderboardRouter />
            <Route path=path!("register") view=Register />
            <Route path=path!("login") view=Login />
            <UserRouter />
        </Routes>
    }
}

#[component(transparent)]
fn UserRouter() -> impl MatchNestedRoutes + Clone {
    let user = expect_context::<UserResource>();
    view! {
        <ProtectedParentRoute
            path=path!("user/@me/dashboard")
            condition=move || user.get().map(|n| n.is_ok())
            redirect_path=|| "/login?redirect=user/@me/dashboard"
            view=Dashboard
        >
            <Route path=path!("") view=() />
            <Route path=path!("username") view=Username />
            <Route path=path!("password") view=Password />
            <Route path=path!("bio") view=Bio />
            <Route path=path!("avatar") view=Avatar />
            <Route path=path!("discord") view=DiscordList />
        </ProtectedParentRoute>
        <ProtectedParentRoute
            path=path!("user/@me/manage")
            condition=move || user.get().map(|n| n.is_ok())
            redirect_path=|| "/login?redirect=user/@me/manage"
            view=ManageRuns
        >
            <Route path=path!("") view=() />
            <Route path=path!(":id") view=Delete />
        </ProtectedParentRoute>
        <ProtectedRoute
            path=path!("user/@me/submit")
            condition=move || user.get().map(|n| n.is_ok())
            redirect_path=|| "/login?redirect=user/@me/submit"
            view=Submit
        />
    }
    .into_inner()
}

#[component(transparent)]
fn LeaderboardRouter() -> impl MatchNestedRoutes + Clone {
    let (patch, set_patch) = signal(String::new());
    let (layout, set_layout) = signal(String::new());
    let (category, set_category) = signal(String::new());
    let (map, set_map) = signal::<Option<String>>(None);

    view! {
        <ParentRoute
            path=path!("leaderboard")
            view=move || view! { <Section patch layout category map /> }
        >
            <Route
                path=path!(":patch/:layout/:category/:map")
                view=move || {
                    view! {
                        <Map patch=set_patch layout=set_layout category=set_category map=set_map />
                    }
                }
            />
            <Route
                path=path!(":patch/:layout/:category")
                view=move || {
                    view! {
                        <Leaderboard
                            patch=set_patch
                            layout=set_layout
                            category=set_category
                            map=set_map
                        />
                    }
                }
            />
            <Route
                path=path!("")
                view=|| {
                    let mut options = NavigateOptions::default();
                    options.replace = true;
                    view! { <Redirect path="2.13/1/standard" options /> }
                }
            />
            <Route
                path=path!(":patch")
                view=|| {
                    let mut options = NavigateOptions::default();
                    options.replace = true;
                    view! { <Redirect path="1/standard" options /> }
                }
            />
            <Route
                path=path!(":patch/:layout")
                view=|| {
                    let mut options = NavigateOptions::default();
                    options.replace = true;
                    view! { <Redirect path="standard" options /> }
                }
            />
        </ParentRoute>
        <ParentRoute
            path=path!("user/:id/leaderboard")
            view=move || {
                view! {
                    <Profile />
                    <Section patch layout category map />
                }
            }
        >
            <Route
                path=path!(":patch/:layout/:category/:map")
                view=move || {
                    view! {
                        <Map patch=set_patch layout=set_layout category=set_category map=set_map />
                    }
                }
            />
            <Route
                path=path!(":patch/:layout/:category")
                view=move || {
                    view! {
                        <Leaderboard
                            patch=set_patch
                            layout=set_layout
                            category=set_category
                            map=set_map
                        />
                    }
                }
            />
            <Route
                path=path!("")
                view=|| {
                    let mut options = NavigateOptions::default();
                    options.replace = true;
                    view! { <Redirect path="2.13/1/standard" options /> }
                }
            />
            <Route
                path=path!(":patch")
                view=|| {
                    let mut options = NavigateOptions::default();
                    options.replace = true;
                    view! { <Redirect path="1/standard" options /> }
                }
            />
            <Route
                path=path!(":patch/:layout")
                view=|| {
                    let mut options = NavigateOptions::default();
                    options.replace = true;
                    view! { <Redirect path="standard" options /> }
                }
            />
        </ParentRoute>
    }
    .into_inner()
}
