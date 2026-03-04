use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use post_archiver::{Alias, Author, AuthorId, PlatformId, query::Totalled, utils::AsTable};

use crate::api::{
    AppState,
    category::{get_category_handler, list_category_handler},
    relation::{RequireRelations, WithRelations},
};

use super::{Category, CategoryOrderBy};

impl RequireRelations for Author {
    fn file_metas(&self) -> Vec<post_archiver::FileMetaId> {
        self.thumb.into_iter().collect()
    }
}

impl Category for Author {
    type Id = AuthorId;
    const DEFAULT_ORDER_BY: CategoryOrderBy = CategoryOrderBy::Updated;

    fn wrap_category_route(router: Router<AppState>) -> Router<AppState> {
        router
            .route(
                &format!("/{}", Self::TABLE_NAME),
                get(list_category_handler::<Self>),
            )
            .route(
                &format!("/{}/{{id}}", Self::TABLE_NAME),
                get(get_category_handler::<Self>),
            )
            .route(
                &format!("/{}/{{id}}/aliases", Self::TABLE_NAME),
                get(author_aliases_handler),
            )
    }
}

pub async fn author_aliases_handler(
    State(state): State<AppState>,
    Path(id): Path<AuthorId>,
) -> Result<Json<WithRelations<Totalled<Vec<Alias>>>>, StatusCode> {
    let manager = &state.manager();
    let items = manager
        .list_author_aliases(id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total = items.len() as u64;

    WithRelations::new(manager, Totalled { items, total })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json::from)
}

impl RequireRelations for Alias {
    fn platforms(&self) -> Vec<PlatformId> {
        vec![self.platform]
    }
}
