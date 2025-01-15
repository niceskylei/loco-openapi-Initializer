#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use loco_rs::prelude::*;
use serde::Serialize;

#[derive(Serialize, Debug, ToSchema)]
pub struct Album {
    title: String,
    rating: u32,
}

/// Get album
///
/// Returns a title and rating
#[utoipa::path(
    get,
    path = "/api/album/get_album",
    tags = ["album"],
    responses(
        (status = 200, description = "Album found", body = Album),
    ),
)]
#[debug_handler]
pub async fn get_album(State(_ctx): State<AppContext>) -> Result<Response> {
    format::json(Album {
        title: "VH II".to_string(),
        rating: 10,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/album/")
        .add("/get_album", routes!(get_album))
}
