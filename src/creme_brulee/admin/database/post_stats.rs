use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "post_stats")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub post_id: Uuid,
    pub view_count: i32,
    pub unique_visitors: i32,
    pub last_viewed: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::blog_posts::Entity",
        from = "Column::PostId",
        to = "super::blog_posts::Column::Id"
    )]
    BlogPost,
}

impl Related<super::blog_posts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BlogPost.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
