use crate::api::*;
use leptos::{
    prelude::*,
    tachys::view::{fragment::IntoFragment, iterators::StaticVec},
};
use web_sys::FormData;

pub struct ViewFragment(Box<dyn FnOnce() -> Fragment + Send + 'static>);

impl Default for ViewFragment {
    fn default() -> Self {
        Self(Box::new(|| Fragment::new(vec![])))
    }
}

impl<F, C> From<F> for ViewFragment
where
    F: FnOnce() -> C + Send + 'static,
    C: IntoFragment + Send + 'static,
{
    fn from(value: F) -> Self {
        Self(Box::new(move || value().into_fragment()))
    }
}

impl ViewFragment {
    /// Execute the wrapped function
    pub fn run(self) -> Fragment {
        (self.0)()
    }
}

pub struct SelectFragment {
    pub nodes: StaticVec<(String, String)>,
}

impl SelectFragment {
    /// Creates a new [`Fragment`].
    #[inline(always)]
    pub fn new(nodes: Vec<(String, String)>) -> Self {
        Self { nodes: nodes.into() }
    }
}

pub trait IntoSelectFragment {
    fn into_select_fragment(self) -> SelectFragment;
}

pub struct ChildrenSelectFragment(Box<dyn FnOnce() -> SelectFragment + Send>);

impl ChildrenSelectFragment {
    pub fn into_inner(self) -> impl FnOnce() -> SelectFragment + Send {
        self.0
    }
}

impl<F> From<F> for SelectFragment
where
    F: IntoSelectFragment,
{
    fn from(value: F) -> Self {
        value.into_select_fragment()
    }
}

impl<F, C> ToChildren<F> for ChildrenSelectFragment
where
    F: FnOnce() -> C + Send + 'static,
    C: IntoSelectFragment,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Self(Box::new(move || f().into_select_fragment()))
    }
}

pub type UserResource = Resource<Result<User, ApiError>>;
pub type UpdatePfpAction = Action<FormData, Result<(), ApiError>>;

#[cfg(feature = "ssr")]
pub type AuthSession = axum_session_auth::AuthSession<User, i64, axum_session_sqlx::SessionPgPool, sqlx::PgPool>;

trait AsString {
    fn to_string(self) -> String;
}

impl AsString for String {
    fn to_string(self) -> String {
        self
    }
}

impl AsString for &str {
    fn to_string(self) -> String {
        ToString::to_string(&self)
    }
}

impl<T> IntoSelectFragment for Vec<(T, T)>
where
    T: ToString,
{
    fn into_select_fragment(self) -> SelectFragment {
        SelectFragment::new(self.into_iter().map(|(v, w)| (v.to_string(), w.to_string())).collect())
    }
}

impl<const N: usize, T> IntoSelectFragment for [(T, T); N]
where
    T: ToString,
{
    fn into_select_fragment(self) -> SelectFragment {
        SelectFragment::new(self.into_iter().map(|(v, w)| (v.to_string(), w.to_string())).collect())
    }
}

macro_rules! tuples {
	($(($ty:ident, $ty2:ident)),*) => {
		impl<$($ty),*,$($ty2),*> IntoSelectFragment for ($(($ty, $ty2)),*)
		where
			$($ty: AsString),*,
            $($ty2: AsString),*,
		{
            fn into_select_fragment(self) -> SelectFragment {
                #[allow(non_snake_case)]
			    let ($(($ty, $ty2)),*) = self;
                SelectFragment::new(vec![$(($ty.to_string(), $ty2.to_string()),)*])
            }
        }
    }
}

tuples!((A, A2));
tuples!((A, A2), (B, B2));
tuples!((A, A2), (B, B2), (C, C2));
tuples!((A, A2), (B, B2), (C, C2), (D, D2));
tuples!((A, A2), (B, B2), (C, C2), (D, D2), (E, E2));
tuples!((A, A2), (B, B2), (C, C2), (D, D2), (E, E2), (F, F2));
tuples!((A, A2), (B, B2), (C, C2), (D, D2), (E, E2), (F, F2), (G, G2));
tuples!((A, A2), (B, B2), (C, C2), (D, D2), (E, E2), (F, F2), (G, G2), (H, H2));
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2),
    (O, O2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2),
    (O, O2),
    (P, P2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2),
    (O, O2),
    (P, P2),
    (Q, Q2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2),
    (O, O2),
    (P, P2),
    (Q, Q2),
    (R, R2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2),
    (O, O2),
    (P, P2),
    (Q, Q2),
    (R, R2),
    (S, S2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2),
    (O, O2),
    (P, P2),
    (Q, Q2),
    (R, R2),
    (S, S2),
    (T, T2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2),
    (O, O2),
    (P, P2),
    (Q, Q2),
    (R, R2),
    (S, S2),
    (T, T2),
    (U, U2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2),
    (O, O2),
    (P, P2),
    (Q, Q2),
    (R, R2),
    (S, S2),
    (T, T2),
    (U, U2),
    (V, V2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2),
    (O, O2),
    (P, P2),
    (Q, Q2),
    (R, R2),
    (S, S2),
    (T, T2),
    (U, U2),
    (V, V2),
    (W, W2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2),
    (O, O2),
    (P, P2),
    (Q, Q2),
    (R, R2),
    (S, S2),
    (T, T2),
    (U, U2),
    (V, V2),
    (W, W2),
    (X, X2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2),
    (O, O2),
    (P, P2),
    (Q, Q2),
    (R, R2),
    (S, S2),
    (T, T2),
    (U, U2),
    (V, V2),
    (W, W2),
    (X, X2),
    (Y, Y2)
);
tuples!(
    (A, A2),
    (B, B2),
    (C, C2),
    (D, D2),
    (E, E2),
    (F, F2),
    (G, G2),
    (H, H2),
    (I, I2),
    (J, J2),
    (K, K2),
    (L, L2),
    (M, M2),
    (N, N2),
    (O, O2),
    (P, P2),
    (Q, Q2),
    (R, R2),
    (S, S2),
    (T, T2),
    (U, U2),
    (V, V2),
    (W, W2),
    (X, X2),
    (Y, Y2),
    (Z, Z2)
);
