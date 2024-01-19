use std::error::Error;

use async_recursion::async_recursion;
use sqlx::Postgres;
use uuid::Uuid;

use crate::models::card::{
    Card, CreateCard, CreateGroup, Group, GroupItems, TreeNode, UpdateCard, UpdateGroup,
};

/// create card in database
pub async fn create_card_db(
    card: &CreateCard,
    pool: &sqlx::Pool<Postgres>,
) -> Result<i32, Box<dyn Error>> {
    let new_card_id = sqlx::query!(
        "INSERT INTO cards (word, translation, group_id) VALUES ($1, $2, $3) RETURNING id",
        card.word,
        card.translation,
        card.group_id,
    )
    .fetch_one(pool)
    .await?
    .id;

    Ok(new_card_id)
}

/// create group in database
pub async fn create_group_db(
    group: &CreateGroup,
    pool: &sqlx::Pool<Postgres>,
) -> Result<i32, Box<dyn Error>> {
    let invite_code = Uuid::new_v4().to_string();

    let new_group_id = sqlx::query!(
        "INSERT INTO card_group (title, group_id, invite_code) VALUES ($1, $2, $3) RETURNING id",
        group.title,
        group.group_id,
        invite_code,
    )
    .fetch_one(pool)
    .await?
    .id;

    Ok(new_group_id)
}

/// get root group for user
pub async fn find_user_root_group(
    user_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<i32, Box<dyn Error>> {
    let id = sqlx::query!(
        "SELECT group_id FROM group_user WHERE user_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?
    .group_id
    .unwrap();

    Ok(id)
}

/// Find root group by other group
pub async fn find_root(group_id: i32, pool: &sqlx::Pool<Postgres>) -> Result<i32, Box<dyn Error>> {
    let root_id: i32 = sqlx::query!(
        r#"
        WITH RECURSIVE Tree AS (
            SELECT id, title, group_id, created_at, updated_at
            FROM card_group
            WHERE id = $1

            UNION ALL

            SELECT t.id, t.title, t.group_id, t.created_at, t.updated_at
            FROM card_group t
            JOIN Tree ON t.id = Tree.group_id
        )
        SELECT id, title, group_id, created_at, updated_at
            FROM Tree
            WHERE group_id IS NULL;
        "#,
        group_id,
    )
    .fetch_one(pool)
    .await?
    .id
    .unwrap();

    Ok(root_id)
}

/// Return true if user is owner of group
pub async fn user_is_owner_group(
    user_id: i32,
    group_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<bool, Box<dyn Error>> {
    let res = sqlx::query!(
        "SELECT id FROM group_user WHERE user_id = $1 AND group_id = $2",
        user_id,
        group_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(res.is_some())
}

/// Get all object in group
pub async fn get_all_objects(
    group_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<Vec<GroupItems>, Box<dyn Error>> {
    let cards = sqlx::query_as!(
        Card,
        "SELECT id, created_at, updated_at, word, translation, group_id FROM cards WHERE group_id = $1",
        group_id
    )
    .fetch_all(pool)
    .await?;

    let groups = sqlx::query_as!(
        Group,
        "SELECT id, created_at, updated_at, title, invite_code, group_id FROM card_group WHERE group_id = $1",
        group_id,
    )
    .fetch_all(pool)
    .await?;

    let mut res = Vec::new();

    for card in cards {
        res.push(GroupItems::Card(card));
    }

    for group in groups {
        res.push(GroupItems::Group(group));
    }

    Ok(res)
}

/// Get only cards in group
pub async fn get_all_cards(
    group_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<Vec<Card>, Box<dyn Error>> {
    let cards = sqlx::query_as!(
        Card,
        "SELECT id, created_at, updated_at, word, translation, group_id FROM cards WHERE group_id = $1",
        group_id
    )
    .fetch_all(pool)
    .await?;

    Ok(cards)
}

/// Get only groups in group
pub async fn get_all_groups(
    group_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<Vec<Group>, Box<dyn Error>> {
    let cards = sqlx::query_as!(
        Group,
        "SELECT id, created_at, updated_at, title, invite_code, group_id FROM card_group WHERE group_id = $1",
        group_id
    )
    .fetch_all(pool)
    .await?;

    Ok(cards)
}

/// Delete the card from database
pub async fn delete_card_db(
    card_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!("DELETE FROM cards WHERE id = $1", card_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Delete the group from database
pub async fn delete_group_db(
    group_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!("DELETE FROM card_group WHERE id = $1", group_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Get card by id from database
pub async fn find_card_by_id(
    card_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<Card, Box<dyn Error>> {
    let card = sqlx::query_as!(
        Card,
        "SELECT id, created_at, updated_at, word, translation, group_id FROM cards WHERE id = $1",
        card_id
    )
    .fetch_one(pool)
    .await?;

    Ok(card)
}

/// Get group by id from database
pub async fn find_group_by_id(
    group_id: i32,
    pool: &sqlx::Pool<Postgres>,
) -> Result<Group, Box<dyn Error>> {
    let group = sqlx::query_as!(
        Group,
        "SELECT id, created_at, updated_at, title, invite_code, group_id FROM card_group WHERE id = $1",
        group_id
    )
    .fetch_one(pool)
    .await?;

    Ok(group)
}

/// Update tne card in database
pub async fn update_card_db(
    card: &UpdateCard,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "UPDATE cards SET word = $2, translation = $3, group_id = $4 WHERE id = $1",
        card.id,
        card.word,
        card.translation,
        card.group_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Update tne group in database
pub async fn update_group_db(
    group: &UpdateGroup,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "UPDATE card_group SET title = $2, group_id = $3 WHERE id = $1",
        group.id,
        group.title,
        group.group_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Find group by invite link in database
pub async fn find_group_by_invite_code(
    invite_code: &str,
    pool: &sqlx::Pool<Postgres>,
) -> Result<Group, Box<dyn Error>> {
    let group = sqlx::query_as!(
        Group,
        "SELECT id, created_at, updated_at, title, invite_code, group_id FROM card_group WHERE invite_code = $1",
        invite_code,
    )
    .fetch_one(pool)
    .await?;

    Ok(group)
}

#[async_recursion]
pub async fn copy_items_recurive(
    orig_id: i32,
    copy_id: i32,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<(), Box<dyn std::error::Error>> {
    let items = get_all_objects(orig_id, pool).await?;

    println!("items: {:#?}", items);

    for item in items {
        match item {
            GroupItems::Card(card) => {
                // create the card in new group
                println!("copy card into group");
                create_card_db(
                    &CreateCard {
                        word: card.word,
                        translation: card.translation,
                        group_id: copy_id,
                    },
                    pool,
                )
                .await?;
            }
            GroupItems::Group(group) => {
                // create the group in new group
                println!("copy group into group");
                let new_group_id = create_group_db(
                    &CreateGroup {
                        title: group.title,
                        group_id: Some(copy_id),
                    },
                    pool,
                )
                .await?;

                // create items in group recursive
                copy_items_recurive(group.id, new_group_id, pool).await?;
            }
        }
    }

    Ok(())
}

/// Return true is user is owner
/// false if user is not or if get some errors
pub async fn user_is_owner_item(
    user_id: i32,
    group_id: i32,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> bool {
    match find_root(group_id, pool).await {
        Ok(root_id) => match user_is_owner_group(user_id, root_id, pool).await {
            Ok(res) => res,
            Err(_) => false,
        },
        Err(_) => false,
    }
}

/// Рекурсивная функция для обхода дерева и извлечения всех вложенных групп
#[async_recursion]
pub async fn get_tree(
    root_id: i32,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<TreeNode, Box<dyn std::error::Error>> {
    // Здесь предполагается, что вы уже имеете список дочерних групп для текущего узла
    let children_groups: Vec<Group> = get_all_groups(root_id, pool).await?;

    // Рекурсивно вызывать эту функцию для каждой дочерней группы
    let mut children_nodes = Vec::new();

    for child_group in children_groups {
        let child_tree = get_tree(child_group.id, pool).await?;
        children_nodes.push(child_tree);
    }

    // Возвращаем узел с текущей группой и её дочерними узлами
    let root_group = find_group_by_id(root_id, pool).await?; // Получите данные для текущей группы
    Ok(TreeNode {
        root: root_group,
        children: children_nodes,
    })
}

/*

///
pub async fn query(pool: &sqlx::Pool<Postgres>) -> Result<(), Box<dyn Error>> {
    Ok(())
}

*/
