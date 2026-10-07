use axum::Router;
use axum::routing::{get, post};

use crate::api::categories::category_controller::{
    CategoryController, create_category, delete_category, get_category, list_categories,
    update_category,
};

/// Tabella delle rotte delle categorie: `/api/categories` e `/api/categories/{id}`.
///
/// Gli handler sono documentati in OpenAPI e leggono il [`CategoryController`]
/// dallo stato di axum.
pub fn router(controller: CategoryController) -> Router {
    Router::new()
        .route("/", post(create_category).get(list_categories))
        .route(
            "/{id}",
            get(get_category)
                .put(update_category)
                .delete(delete_category),
        )
        .with_state(controller)
}
