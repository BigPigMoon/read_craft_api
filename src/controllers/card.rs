use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{extractors::jwt_cred::JwtCred, models::card::*, services::card::*, AppState};

pub fn card_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/card")
            .service(create_card)
            .service(get_cards_in_group)
            .service(delete_card)
            .service(update_card),
    );
}

/// Create the card by JSON [CreateCard] return id of new card
///
/// Path:
/// POST: /api/card/create
#[post("/create")]
async fn create_card(
    creds: JwtCred,
    card: web::Json<CreateCard>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "create_card";

    let user_id = creds.uid;

    log::info!("{}: attemting to create card", op);

    if !user_is_owner_item(user_id, card.group_id, &app_data.pool).await {
        log::warn!(
            "{}: user: {}, is not owner of group: {}",
            op,
            user_id,
            card.group_id
        );

        return HttpResponse::Forbidden().finish();
    }

    let id = match create_card_db(&card, &app_data.pool).await {
        Ok(id) => id,
        Err(err) => {
            log::error!("{}: can not create card, error: {}", op, err);

            return HttpResponse::InternalServerError().finish();
        }
    };

    log::info!("{}: card are successfuly creating: {}", op, id);

    HttpResponse::Ok().json(id)
}

/// Get all cards in group by id return [Vec] of [Card]
///
/// Path:
/// GET: /api/card/all/{group_id}
#[get("/all/{id}")]
async fn get_cards_in_group(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "get_cards_in_group";

    let user_id = creds.uid;
    let group_id = path.into_inner();

    log::info!("{}: attemting to get cards in group: {}", op, group_id);

    if !user_is_owner_item(user_id, group_id, &app_data.pool).await {
        log::warn!(
            "{}: user: {}, is not owner of group: {}",
            op,
            user_id,
            group_id
        );

        return HttpResponse::Forbidden().finish();
    }

    match get_all_cards(group_id, &app_data.pool).await {
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

/// Delete the card by id
///
/// Path:
/// DELETE: /api/card/delete/{card_id}
#[delete("/delete/{id}")]
async fn delete_card(
    creds: JwtCred,
    path: web::Path<i32>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "delete_card";

    let user_id = creds.uid;
    let card_id = path.into_inner();

    log::info!("{}: attempting to delete the card: {}", op, card_id);

    let card = match find_card_by_id(card_id, &app_data.pool).await {
        Ok(card) => card,
        Err(err) => {
            log::error!("{}: can not find card: {}, error: {}", op, card_id, err);

            return HttpResponse::NotFound().finish();
        }
    };

    if !user_is_owner_item(user_id, card.group_id.unwrap(), &app_data.pool).await {
        log::warn!(
            "{}: user: {}, is not owner of group: {}",
            op,
            user_id,
            card.group_id.unwrap()
        );

        return HttpResponse::Forbidden().finish();
    }

    match delete_card_db(card_id, &app_data.pool).await {
        Ok(_) => {
            log::info!("{}: card is successfuly deleted", op);

            HttpResponse::Ok().finish()
        }
        Err(err) => {
            log::error!("{}: can not delete card: {}, error: {}", op, card_id, err);

            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Update the card by JSON [UpdateCard]
///
/// Path:
/// PUT: /api/card/update
#[put("/update")]
async fn update_card(
    creds: JwtCred,
    new_card: web::Json<UpdateCard>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "update_card";

    let user_id = creds.uid;
    let card_id = new_card.id;

    log::info!("{}: attempting to delete the card: {}", op, card_id);

    let card = match find_card_by_id(card_id, &app_data.pool).await {
        Ok(card) => card,
        Err(err) => {
            log::error!("{}: can not find card: {}, error: {}", op, card_id, err);

            return HttpResponse::NotFound().finish();
        }
    };

    if !user_is_owner_item(user_id, card.group_id.unwrap(), &app_data.pool).await {
        log::warn!(
            "{}: user: {}, is not owner of group: {}",
            op,
            user_id,
            card.group_id.unwrap()
        );

        return HttpResponse::Forbidden().finish();
    }

    match update_card_db(&new_card, &app_data.pool).await {
        Ok(_) => {
            log::info!("{}: card are successfuly updated", op);

            HttpResponse::Ok().finish()
        }
        Err(err) => {
            log::error!("{}: can not update the card, error: {}", op, err);

            HttpResponse::InternalServerError().finish()
        }
    }
}
