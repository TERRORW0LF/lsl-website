use leptos::{either::*, prelude::*};
use leptos_meta::MetaTags;
use leptos_meta::*;
use leptos_router::{
    MatchNestedRoutes, NavigateOptions,
    any_nested_route::IntoAnyNestedRoute,
    components::{
        A, Outlet, ParentRoute, ProtectedParentRoute, ProtectedRoute, Redirect, Route, Router,
        Routes,
    },
    hooks::use_params_map,
    path,
};
use pages::{
    Activity, ComboRanking, Dashboard, ErrorTemplate, FAQ, HomePage, Leaderboard, Login,
    ManageRuns, Map, Profile, Register, Submit, Submits, UserRanking,
    dash::{Avatar, Bio, DiscordList, Password, Username},
    error_template::AppError,
    leaderboard::Section,
    ranking::RankingHeader,
    user::Delete,
};
use server::auth::{Login, Logout, Register, UpdateBio, UpdateCreds, get_current_user, update_pfp};
use types::leptos::UserResource;
use wasm_bindgen::{JsCast, prelude::Closure};
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
                            <A href="/leaderboard/2.13/1/standard">"Leaderboard"</A>
                        </li>
                        <li>
                            <A href="/ranking/2.13/1">"Ranking"</A>
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
            <Route path=path!("runs") view=Submits />
            <Route path=path!("activity") view=Activity />
            <LeaderboardRouter />
            <RankingRouter />
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
    .into_any_nested_route()
}

#[component(transparent)]
fn LeaderboardRouter() -> impl MatchNestedRoutes + Clone {
    view! {
        <Route
            path=path!("leaderboard")
            view=move || {
                let mut options = NavigateOptions::default();
                options.replace = true;
                view! { <Redirect path="2.13/1/standard" options /> }
            }
        />
        <Route
            path=path!("leaderboard/:patch/:layout/:category")
            view=move || {
                let params = use_params_map();
                let patch = Signal::derive(move || { params.read().get("patch").unwrap() });
                let layout = Signal::derive(move || { params.read().get("layout").unwrap() });
                let category = Signal::derive(move || { params.read().get("category").unwrap() });
                let layouts = Signal::derive(move || match patch.get().as_str() {
                    "1.00" => vec![("1".into(), "Layout 1".into()), ("2".into(), "Layout 2".into())],
                    "1.41" => {
                        vec![
                            ("1".into(), "Layout 1".into()),
                            ("2".into(), "Layout 2".into()),
                            ("3".into(), "Layout 3".into()),
                            ("4".into(), "Layout 4".into()),
                        ]
                    }
                    "1.50" | "2.00" | "2.13" => {
                        vec![
                            ("1".into(), "Layout 1".into()),
                            ("2".into(), "Layout 2".into()),
                            ("3".into(), "Layout 3".into()),
                            ("4".into(), "Layout 4".into()),
                            ("5".into(), "Layout 5".into()),
                        ]
                    }
                    _ => vec![],
                });
                let categories: Vec<(String, String)> = vec![
                    ("standard".into(), "Standard".into()),
                    ("gravspeed".into(), "Gravspeed".into()),
                ];
                view! {
                    <section id="leaderboard">
                        <Section layouts categories category />
                        <Leaderboard patch layout category />
                    </section>
                }
            }
        />
        <Route
            path=path!("leaderboard/map/:map")
            view=move || {
                let params = use_params_map();
                let id = Signal::derive(move || {
                    params.read().get("map").unwrap().parse::<i32>().unwrap_or(0)
                });
                view! {
                    <section id="leaderboard">
                        <Map id />
                    </section>
                }
            }
        />
        <ParentRoute
            path=path!("user/:id")
            view=move || {
                let params = use_params_map();
                let id = Signal::derive(move || {
                    params.read().get("id").unwrap().parse::<i64>().unwrap_or(0)
                });
                view! {
                    <Profile id />
                    <Outlet />
                }
            }
        >
            <Route
                path=path!("leaderboard/:patch/:layout/:category")
                view=move || {
                    let params = use_params_map();
                    let patch = Signal::derive(move || { params.read().get("patch").unwrap() });
                    let layout = Signal::derive(move || { params.read().get("layout").unwrap() });
                    let category = Signal::derive(move || {
                        params.read().get("category").unwrap()
                    });
                    let layouts = Signal::derive(move || match patch.get().as_str() {
                        "1.00" => {
                            vec![("1".into(), "Layout 1".into()), ("2".into(), "Layout 2".into())]
                        }
                        "1.41" => {
                            vec![
                                ("1".into(), "Layout 1".into()),
                                ("2".into(), "Layout 2".into()),
                                ("3".into(), "Layout 3".into()),
                                ("4".into(), "Layout 4".into()),
                            ]
                        }
                        "1.50" | "2.00" | "2.13" => {
                            vec![
                                ("1".into(), "Layout 1".into()),
                                ("2".into(), "Layout 2".into()),
                                ("3".into(), "Layout 3".into()),
                                ("4".into(), "Layout 4".into()),
                                ("5".into(), "Layout 5".into()),
                            ]
                        }
                        _ => vec![],
                    });
                    let categories: Vec<(String, String)> = vec![
                        ("standard".into(), "Standard".into()),
                        ("gravspeed".into(), "Gravspeed".into()),
                    ];
                    view! {
                        <section id="leaderboard">
                            <Section layouts categories category />
                            <Leaderboard patch layout category />
                        </section>
                    }
                }
            />
            <Route
                path=path!("leaderboard/map/:map")
                view=move || {
                    let params = use_params_map();
                    let id = Signal::derive(move || {
                        params.read().get("map").unwrap().parse::<i32>().unwrap_or(0)
                    });
                    view! {
                        <section id="leaderboard">
                            <Map id />
                        </section>
                    }
                }
            />
            <Route
                path=path!("ranking")
                view=move || {
                    let params = use_params_map();
                    let id = Signal::derive(move || {
                        params.read().get("id").unwrap().parse::<i64>().unwrap_or(0)
                    });
                    let patches: Vec<(String, String)> = vec![
                        ("1.00".into(), "Patch 1.00".into()),
                        ("1.41".into(), "Patch 1.41".into()),
                        ("1.50".into(), "Patch 1.50".into()),
                        ("2.00".into(), "Patch 2.00".into()),
                        ("2.13".into(), "Patch 2.13".into()),
                    ];
                    view! { <UserRanking id patches /> }
                }
            />
            <Route
                path=path!("leaderboard")
                view=move || {
                    let mut options = NavigateOptions::default();
                    options.replace = true;
                    view! { <Redirect path="2.13/1/standard" options /> }
                }
            />
            <Route path=path!("") view=move || view! {} />
        </ParentRoute>
    }
    .into_inner()
    .into_any_nested_route()
}

#[derive(Clone)]
struct Patch(Signal<String>);

#[component(transparent)]
fn RankingRouter() -> impl MatchNestedRoutes + Clone {
    view! {
        <ParentRoute
            path=path!("/ranking")
            view=move || {
                let patches: Vec<(String, String)> = vec![
                    ("1.00".into(), "Patch 1.00".into()),
                    ("1.41".into(), "Patch 1.41".into()),
                    ("1.50".into(), "Patch 1.50".into()),
                    ("2.00".into(), "Patch 2.00".into()),
                    ("2.13".into(), "Patch 2.13".into()),
                ];
                view! {
                    <section id="ranking">
                        <RankingHeader links=patches />
                        <Outlet />
                    </section>
                }
            }
        >
            <Route
                path=path!("")
                view=|| view! { <p>"Please select a patch to view the rankings of."</p> }
            />
            <ParentRoute
                path=path!(":patch")
                view=move || {
                    let params = use_params_map();
                    let patch = Signal::derive(move || { params.get().get("patch").unwrap() });
                    let layouts = Signal::derive(move || match patch.get().as_str() {
                        "1.00" => {
                            vec![("1".into(), "Layout 1".into()), ("2".into(), "Layout 2".into())]
                        }
                        "1.41" => {
                            vec![
                                ("1".into(), "Layout 1".into()),
                                ("2".into(), "Layout 2".into()),
                                ("3".into(), "Layout 3".into()),
                                ("4".into(), "Layout 4".into()),
                            ]
                        }
                        "1.50" | "2.00" | "2.13" => {
                            vec![
                                ("1".into(), "Layout 1".into()),
                                ("2".into(), "Layout 2".into()),
                                ("3".into(), "Layout 3".into()),
                                ("4".into(), "Layout 4".into()),
                                ("5".into(), "Layout 5".into()),
                            ]
                        }
                        _ => vec![],
                    });
                    provide_context(Patch(patch));
                    view! {
                        <ComboRanking
                            patch=patch
                            layout=None
                            categories=vec![(None, "Combined".into())]
                        />
                        <RankingHeader links=layouts />
                        <Outlet />
                    }
                }
            >
                <Route
                    path=path!(":layout")
                    view=move || {
                        let params = use_params_map();
                        let patch = expect_context::<Patch>().0;
                        let layout = Signal::derive(move || { params.read().get("layout") });
                        let categories = Signal::derive(move || match patch.get().as_str() {
                            "1.00" | "1.41" | "1.50" | "2.00" => {
                                vec![
                                    (Some("Standard".into()), "Standard".into()),
                                    (Some("Gravspeed".into()), "Gravspeed".into()),
                                    (None, "Combined".into()),
                                ]
                            }
                            "2.13" => {
                                vec![
                                    (Some("Standard".into()), "Standard".into()),
                                    (Some("Gravspeed".into()), "Gravspeed".into()),
                                ]
                            }
                            _ => vec![],
                        });
                        view! { <ComboRanking patch layout categories /> }
                    }
                />
                <Route
                    path=path!("")
                    view=|| {
                        view! { <p>"Please select a layout to view the rankings of."</p> }
                    }
                />
            </ParentRoute>
        </ParentRoute>
    }
    .into_inner()
    .into_any_nested_route()
}
