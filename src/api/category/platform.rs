use post_archiver::{Platform, PlatformId};

use crate::api::relation::RequireRelations;

use super::Category;

impl RequireRelations for Platform {}

impl Category for Platform {
    type Id = PlatformId;
}
