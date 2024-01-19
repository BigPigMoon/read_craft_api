use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::{
    extractors::jwt_cred::JwtCred,
    models::{card::*, common::ErrorResponse},
    services::card::*,
    AppState,
};

pub fn group_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/group")
            .service(create_group)
            .service(get_items_in_group)
            .service(get_root_for_user)
            .service(delete_group)
            .service(update_group)
            .service(copy_group)
            .service(get_path_to_group)
            .service(get_full_tree),
    );
}

/// Create the group by JSON [CreateGroup] return id of new group
///
/// Path:
/// POST: /api/group/create
#[post("/create")]
async fn create_group(
    creds: JwtCred,
    group: web::Json<CreateGroup>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "create_group";

    let user_id = creds.uid;

    log::info!("{}: attempting to create group", op);

    if group.group_id.is_none() {
        log::warn!(
            "{}: user: {}, is not owner of group: {}",
            op,
            user_id,
            group.group_id.unwrap()
        );

        return HttpResponse::BadRequest().json(ErrorResponse {
            message: "group id shoul be set".to_string(),
        });
    }

    if !user_is_owner_item(user_id, group.group_id.unwrap(), &app_data.pool).await {
        log::warn!(
            "{}: user: {}, is not owner of group: {:?}",
            op,
            user_id,
            group.group_id
        );

        return HttpResponse::Forbidden().finish();
    }

    let id = match create_group_db(&group, &app_data.pool).await {
        Ok(id) => id,
        Err(err) => {
            log::error!("{}: can not create group, error: {}", op, err);

            return HttpResponse::InternalServerError().finish();
        }
    };

    log::info!("{}: group are successfuly creating: {}", op, id);

    HttpResponse::Ok().json(id)
}

/// Get the id of root group for user
///
/// Path:
/// GET: /api/group/root
#[get("/root")]
async fn get_root_for_user(creds: JwtCred, app_data: web::Data<AppState>) -> impl Responder {
    let op = "get_root_for_user";

    let user_id = creds.uid;

    log::info!(
        "{}: attempting to get root of group for user: {}",
        op,
        user_id
    );

    let id = match find_user_root_group(user_id, &app_data.pool).await {
        Ok(id) => id,
        Err(err) => {
            log::error!("{}: can not get id of root group, error: {}", op, err);

            return HttpResponse::InternalServerError().finish();
        }
    };

    log::info!(
        "{}: root of group for user: {} are successfuly returned: root_id: {}",
        op,
        user_id,
        id
    );

    HttpResponse::Ok().json(id)
}

/// Get object in group return Vec of [GroupItems]
///
/// [GroupItems] is enum that is combinati of [Card] and [Group]
///
/// Path:
/// GET: /api/group/items/{group_id}
#[get("/items/{id}")]
async fn get_items_in_group(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "get_objects_in_group";

    let user_id = creds.uid;
    let group_id = path.into_inner();

    log::info!("{}: attemting to get items in group: {}", op, group_id);

    if !user_is_owner_item(user_id, group_id, &app_data.pool).await {
        log::warn!(
            "{}: user: {}, is not owner of group: {}",
            op,
            user_id,
            group_id
        );

        return HttpResponse::Forbidden().finish();
    }

    match get_all_objects(group_id, &app_data.pool).await {
        Ok(items) => {
            log::info!(
                "{}: items from group: {} are successfuly returned",
                op,
                group_id
            );

            HttpResponse::Ok().json(items)
        }
        Err(err) => {
            log::error!(
                "{}: can not get all items in group: {}, error: {}",
                op,
                group_id,
                err
            );

            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Delete the group by id
///
/// Path:
/// DELETE: /api/group/delete/{card_id}
#[delete("/delete/{id}")]
async fn delete_group(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "delete_group";

    let user_id = creds.uid;
    let group_id = path.into_inner();

    log::info!("{}: attempting to delete the group: {}", op, group_id);

    let group = match find_group_by_id(group_id, &app_data.pool).await {
        Ok(card) => card,
        Err(err) => {
            log::error!("{}: can not find card: {}, error: {}", op, group_id, err);

            return HttpResponse::NotFound().finish();
        }
    };

    if group.group_id.is_none() {
        log::warn!(
            "{}: can not delete the root of groups, group_id: {}",
            op,
            group_id
        );

        return HttpResponse::BadRequest().json(ErrorResponse {
            message: "can not delete root".to_string(),
        });
    }

    if !user_is_owner_item(user_id, group.group_id.unwrap(), &app_data.pool).await {
        log::warn!(
            "{}: user: {}, is not owner of group: {}",
            op,
            user_id,
            group.group_id.unwrap()
        );

        return HttpResponse::Forbidden().finish();
    }

    match delete_group_db(group_id, &app_data.pool).await {
        Ok(_) => {
            log::info!("{}: group is successfuly deleted", op);

            HttpResponse::Ok().finish()
        }
        Err(err) => {
            log::error!("{}: can not delete group: {}, error: {}", op, group_id, err);

            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Update the group by JSON [UpdateGroup]
///
/// Path:
/// PUT: /api/group/update
#[put("/update")]
async fn update_group(
    creds: JwtCred,
    new_group: web::Json<UpdateGroup>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "update_group";

    let user_id = creds.uid;
    let card_id = new_group.id;

    if new_group.id == new_group.group_id.unwrap() {
        log::warn!("{}: do not move folder to folder", op);

        return HttpResponse::BadRequest().finish();
    }

    log::info!("{}: attempting to delete the card: {}", op, card_id);

    let group = match find_group_by_id(card_id, &app_data.pool).await {
        Ok(card) => card,
        Err(err) => {
            log::error!("{}: can not find card: {}, error: {}", op, card_id, err);

            return HttpResponse::NotFound().finish();
        }
    };

    if !user_is_owner_item(user_id, group.group_id.unwrap(), &app_data.pool).await {
        log::warn!(
            "{}: user: {}, is not owner of group: {}",
            op,
            user_id,
            group.group_id.unwrap()
        );

        return HttpResponse::Forbidden().finish();
    }

    match update_group_db(&new_group, &app_data.pool).await {
        Ok(_) => {
            log::info!("{}: group are successfuly updated", op);

            HttpResponse::Ok().finish()
        }
        Err(err) => {
            log::error!("{}: can not update the card, error: {}", op, err);

            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
pub struct CopyGroup {
    invite_code: String,
    parent_id: i32,
}

/// Copy gropu by invite code to parent id group use JSON [CopyGroup]
///
/// Path:
/// POST: /api/group/copy
#[post("/copy")]
async fn copy_group(
    creds: JwtCred,
    copy_option: web::Json<CopyGroup>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "copy_group";

    let user_id = creds.uid;

    log::info!(
        "{}: attemting to copy group into: {}",
        op,
        copy_option.parent_id
    );

    if !user_is_owner_item(user_id, copy_option.parent_id, &app_data.pool).await {
        log::warn!(
            "{}: user: {}, is not owner of group: {}",
            op,
            user_id,
            copy_option.parent_id
        );

        return HttpResponse::Forbidden().finish();
    }

    let group_copy = match find_group_by_invite_code(&copy_option.invite_code, &app_data.pool).await
    {
        Ok(group) => group,
        Err(err) => {
            log::error!("{}: can not find group by invite code, error: {}", op, err);

            return HttpResponse::NotFound().finish();
        }
    };

    if let Err(err) =
        copy_items_recurive(group_copy.id, copy_option.parent_id, &app_data.pool).await
    {
        log::error!("{}: can not create copy of group, error: {}", op, err);

        return HttpResponse::InternalServerError().finish();
    }

    log::info!("{}: group are successfuly copied", op);

    HttpResponse::Ok().finish()
}

/// Get path to group return [Vec] of [Group]
///
/// Path:
/// GET: /api/group/path/{group_id}
#[get("/path/{id}")]
async fn get_path_to_group(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "get_path_to_group";

    let user_id = creds.uid;
    let target_group_id = path.into_inner();

    log::info!(
        "{}: attempting to get full path to: {}",
        op,
        target_group_id
    );

    let mut group = match find_group_by_id(target_group_id, &app_data.pool).await {
        Ok(card) => card,
        Err(err) => {
            log::error!(
                "{}: can not find card: {}, error: {}",
                op,
                target_group_id,
                err
            );

            return HttpResponse::NotFound().finish();
        }
    };

    if !user_is_owner_item(user_id, group.id, &app_data.pool).await {
        log::warn!(
            "{}: user: {}, is not owner of group: {}",
            op,
            user_id,
            group.group_id.unwrap()
        );

        return HttpResponse::Forbidden().finish();
    }

    let mut res: Vec<Group> = Vec::new();

    while let Some(parent_id) = group.group_id {
        res.push(group);

        group = find_group_by_id(parent_id, &app_data.pool).await.unwrap();
    }

    res.push(group);

    log::info!("{}: path successfuly getted: {:?}", op, res);

    HttpResponse::Ok().json(res)
}

/// Get full tree of folder by user
///
/// Path:
/// GET: /api/group/tree
#[get("/tree")]
async fn get_full_tree(creds: JwtCred, app_data: web::Data<AppState>) -> impl Responder {
    let op = "get_full_tree";

    let user_id = creds.uid;

    log::info!("{}: attempting to get tree for user: {}", op, user_id);

    let root_id = match find_user_root_group(user_id, &app_data.pool).await {
        Ok(id) => id,
        Err(err) => {
            log::error!("{}: can not get id of root group, error: {}", op, err);

            return HttpResponse::InternalServerError().finish();
        }
    };

    let tree = match get_tree(root_id, &app_data.pool).await {
        Ok(tree) => tree,
        Err(err) => {
            log::error!("{}: can not get full tree, error: {}", op, err);

            return HttpResponse::InternalServerError().finish();
        }
    };

    log::info!(
        "{}: tree for user: {} are successfuly returned",
        op,
        user_id,
    );

    HttpResponse::Ok().json(tree)
}
