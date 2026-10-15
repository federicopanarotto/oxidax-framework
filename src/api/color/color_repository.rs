use async_trait::async_trait;

use crate::api::color::color_entity::Color;
use crate::api::color::color_response::ColorResponse;
use crate::errors::AppResult;
use crate::framework::ViewRepository;

/// Persistence contract for colors.
///
/// It inherits the populated reads (`find`, `find_page`, `count`, `delete`)
/// from [`ViewRepository`]; only the aggregate-specific writes are declared
/// here.
#[async_trait]
pub trait ColorRepository: ViewRepository<Response = ColorResponse> {
    async fn insert(&self, item: Color) -> AppResult<ColorResponse>;
    async fn replace(&self, id: &str, item: Color) -> AppResult<Option<ColorResponse>>;
}
