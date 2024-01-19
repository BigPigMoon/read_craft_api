use std::{sync::MutexGuard, time::Duration};

use actix_web::{get, web, HttpResponse, Responder};
use redis::{Commands, Connection};
use serde::Deserialize;

use crate::{
    extractors::jwt_cred::JwtCred,
    models::{language::Language, translator::WordData},
    AppState,
};

pub fn trasnlator_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/translator")
            .service(get_trasnation_word)
            .service(get_trasnation_words),
    );
}

#[derive(Debug, Deserialize)]
struct GetTranlationOptions {
    query: String,
    src: Language,
    dst: Language,
}

#[get("/word")]
async fn get_trasnation_word(
    _: JwtCred,
    q: web::Query<GetTranlationOptions>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "get_trasnation_word";
    let mut redis_conn = app_data.redis.lock().unwrap();

    log::info!("{}: attemting to get translation of word: {:#?}", op, q);

    match get_word_translation(&q.0, &mut redis_conn) {
        Ok(trans_json) => {
            log::info!(
                "{}: translation of word successfuly returned, translations",
                op,
            );

            HttpResponse::Ok().json(trans_json)
        }
        Err(err) => {
            log::error!("{}: can not get translation fo word, error: {}", op, err);

            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/words")]
async fn get_trasnation_words(
    _: JwtCred,
    q: web::Query<GetTranlationOptions>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    let op = "get_trasnation_words";
    let mut redis_conn = app_data.redis.lock().unwrap();

    log::info!("{}: attemting to get translation of words: {:#?}", op, q);

    let mut res = Vec::new();

    for word in q.query.split(' ') {
        let single_word_query = GetTranlationOptions {
            query: word.to_string(),
            src: q.src,
            dst: q.dst,
        };

        if let Ok(translation) = get_word_translation(&single_word_query, &mut redis_conn) {
            res.push(translation);
        };
    }

    log::info!("{}: words are successfuly translated", op);

    HttpResponse::Ok().json(res)
}

// FIXME: need to by async request to redis and linguee api
fn get_word_translation(
    query: &GetTranlationOptions,
    redis_conn: &mut MutexGuard<'_, Connection>,
) -> Result<WordData, Box<dyn std::error::Error>> {
    let op = "get_word_translation";

    // try to find translation in redis and return
    let cache_key = format!("translator:{}:{}:{}", query.query, query.src, query.dst);

    if let Ok(cached_translation) = redis_conn.get::<&str, String>(&cache_key) {
        let trans_json: WordData = serde_json::from_str(&cached_translation)?;

        log::info!(
            "{}: translation of word successfuly returned from cache",
            op,
        );

        return Ok(trans_json);
    };

    log::info!("{}: word are not cached, do request", op);

    // do request to linguee api
    let trans_json: WordData = ureq::get(
        format!(
            "http://127.0.0.1:8000/api/v2/translations?query={}&src={}&dst={}",
            query.query,
            query.src.to_string().to_lowercase(),
            query.dst.to_string().to_lowercase(),
        )
        .as_str(),
    )
    .call()?
    .into_json()?;

    let json_string = serde_json::to_string(&trans_json).unwrap();

    let one_day_secs = 86400;

    log::info!("{}: translate are successfuly returned from reqest", op);

    // write translation to redis
    let _: () = redis_conn.set_ex(
        &cache_key,
        &json_string,
        Duration::from_secs(one_day_secs * 7).as_secs(),
    )?;

    Ok(trans_json)
}
