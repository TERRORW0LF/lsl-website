use leptos::prelude::*;
use server_fn::codec::GetUrl;
use types::api::ApiError;

#[server(RecalculateRankings, prefix="/api", endpoint="ranking/recalculate", input=GetUrl)]
pub async fn recalculate_ranks() -> Result<(), ApiError> {
    Ok(())
}

#[server(RecalculateRuns, prefix="/api", endpoint="runs/recalculate", input=GetUrl)]
pub async fn recalculate_runs() -> Result<(), ApiError> {
    Ok(())
}

#[server(AddSection, prefix="/api", endpoint="section/add", input=GetUrl)]
pub async fn add_section() -> Result<(), ApiError> {
    Ok(())
}

#[server(UpdateSetion, prefix="/api", endpoint="section/update", input=GetUrl)]
pub async fn update_section() -> Result<(), ApiError> {
    Ok(())
}
