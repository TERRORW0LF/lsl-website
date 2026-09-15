use leptos::prelude::*;
use leptos_router::components::{A, Outlet};

#[component]
pub fn Admin() -> impl IntoView {
    view! {
        <div class="sidebar-container">
            <div class="hamburger" class="sidebar-toggle">
                <input type="checkbox" />
                <span></span>
                <span></span>
                <span></span>
            </div>
            <nav class="sidebar-nav">
                <ul>
                    <li>
                        <A href="runs">"Submits"</A>
                    </li>
                    <li>
                        <A href="sections">"Sections"</A>
                    </li>
                    <li>
                        <A href="users">"Users"</A>
                    </li>
                </ul>
            </nav>
            <main>
                <Outlet />
            </main>
        </div>
    }
}

#[component]
pub fn ManageRuns() -> impl IntoView {
    view! {
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
        <p>"test"</p>
    }
}

#[component]
pub fn ManageSections() -> impl IntoView {
    view! { <h1>"Placeholder"</h1> }
}

#[component]
pub fn ManageUsers() -> impl IntoView {
    view! { <h1>"Placeholder"</h1> }
}
