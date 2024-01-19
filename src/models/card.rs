use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct Card {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub word: String,
    pub translation: String,
    pub group_id: Option<i32>,
}

#[derive(Clone, Debug, Validate, Deserialize, Serialize)]
pub struct CreateCard {
    pub word: String,
    pub translation: String,
    pub group_id: i32,
}

#[derive(Clone, Debug, Validate, Deserialize, Serialize)]
pub struct UpdateCard {
    pub id: i32,
    pub word: String,
    pub translation: String,
    pub group_id: i32,
}

#[derive(Clone, Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct Group {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub title: String,
    pub invite_code: String,
    pub group_id: Option<i32>,
}

#[derive(Clone, Debug, Validate, Deserialize, Serialize)]
pub struct CreateGroup {
    pub title: String,
    pub group_id: Option<i32>,
}

#[derive(Clone, Debug, Validate, Deserialize, Serialize)]
pub struct UpdateGroup {
    pub id: i32,
    pub title: String,
    pub group_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GroupItems {
    Card(Card),
    Group(Group),
}

/// Структура представляющая узел дерева
#[derive(Debug, Deserialize, Serialize)]
pub struct TreeNode {
    pub root: Group,
    pub children: Vec<TreeNode>,
}
