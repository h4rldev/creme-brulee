use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "blog_posts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub post_id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub published: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::post_stats::Entity")]
    PostStats,
}

impl Related<super::post_stats::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PostStats.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
