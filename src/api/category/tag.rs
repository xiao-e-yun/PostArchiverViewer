use post_archiver::{PlatformId, Tag, TagId};

use crate::api::relation::RequireRelations;

use super::Category;

impl RequireRelations for Tag {
    fn platforms(&self) -> Vec<PlatformId> {
        self.platform.into_iter().collect()
    }
}

impl Category for Tag {
    type Id = TagId;
}
