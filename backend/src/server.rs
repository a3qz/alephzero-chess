use std::str::FromStr;
use std::sync::Arc;
#[cfg(feature = "server")]
use async_std::sync::{Mutex, Condvar};
use std::time::Duration;
mod error;
#[cfg(feature = "server")]
use actix_web::{get, HttpResponse};
#[cfg(feature = "server")]
use actix_web::{web, App, HttpServer};
use num_bigint::BigInt;
mod board;
mod board_serializer;
mod pawn_rank;
mod moves;
mod piece;
mod piece_rules;
mod piece_serializer;
use crate::piece::Piece;
/*#[cfg(feature = "server")]
use rusqlite::Connection;*/
use crate::piece_rules::StandardChess;

use crate::board::Board;
#[cfg(feature = "server")]
use crate::board_serializer::board_serialize;
use crate::error::*;
#[cfg(feature = "server")]
use actix_files as fs;
#[cfg(feature = "server")]
use async_std::task;

type SharedData = (Mutex<Board>, Condvar);
type Shared = web::Data<SharedData>;

#[cfg(feature = "server")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut b = Board::new();
    b.place_piece(Piece::new(
        "rook".to_string(),
        piece::Color::Black,
        0.into(),
        0.into(),
    ));
    b.place_piece(Piece::new(
        "rook".to_string(),
        piece::Color::White,
        7.into(),
        0.into(),
    ));
    b.place_piece(Piece::new(
        "knight".to_string(),
        piece::Color::Black,
        0.into(),
        1.into(),
    ));
    b.place_piece(Piece::new(
        "knight".to_string(),
        piece::Color::White,
        7.into(),
        1.into(),
    ));
    b.place_piece(Piece::new(
        "bishop".to_string(),
        piece::Color::Black,
        0.into(),
        2.into(),
    ));
    b.place_piece(Piece::new(
        "bishop".to_string(),
        piece::Color::White,
        7.into(),
        2.into(),
    ));
    b.place_piece(Piece::new(
        "queen".to_string(),
        piece::Color::Black,
        0.into(),
        3.into(),
    ));
    b.place_piece(Piece::new(
        "queen".to_string(),
        piece::Color::White,
        7.into(),
        3.into(),
    ));
    b.place_piece(Piece::new(
        "king".to_string(),
        piece::Color::Black,
        0.into(),
        4.into(),
    ));
    b.place_piece(Piece::new(
        "king".to_string(),
        piece::Color::White,
        7.into(),
        4.into(),
    ));
    b.place_piece(Piece::new(
        "bishop".to_string(),
        piece::Color::Black,
        0.into(),
        5.into(),
    ));
    b.place_piece(Piece::new(
        "bishop".to_string(),
        piece::Color::White,
        7.into(),
        5.into(),
    ));
    b.place_piece(Piece::new(
        "knight".to_string(),
        piece::Color::Black,
        0.into(),
        6.into(),
    ));
    b.place_piece(Piece::new(
        "knight".to_string(),
        piece::Color::White,
        7.into(),
        6.into(),
    ));
    b.place_piece(Piece::new(
        "rook".to_string(),
        piece::Color::Black,
        0.into(),
        7.into(),
    ));
    b.place_piece(Piece::new(
        "rook".to_string(),
        piece::Color::White,
        7.into(),
        7.into(),
    ));
    let board: Shared = web::Data::new((Mutex::new(b), Condvar::new()));
    HttpServer::new(move || {
        App::new()
            .service(get)
            .service(get_legal)
            .service(get_move)
            .service(get_promote)
            .service(get_version)
            .app_data(board.clone())
            .data(Arc::new(StandardChess::new()))
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;
    Ok(())
}

#[get("/board")]
pub async fn get(shared: Shared) -> Result<HttpResponse, Error> {
    let (board, cvar) = &**shared;
    let b = board.lock().await;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(board_serialize(&b)))
}

#[get("/board/{version}")]
pub async fn get_version(shared: Shared, web::Path((version)): web::Path<(String)>) -> Result<HttpResponse, Error> {
    let (board, cvar) = &**shared;
    let version2 = version.parse::<BigInt>().map_err(|_| Error::new())?;
    let mut b = board.lock().await;
    while  b.turn < version2 {
       b = cvar.wait(b).await;
    }
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(board_serialize(&b)))
}

#[get("/legal/{px}/{py}/{wx}/{wy}/{zoom}")]
pub async fn get_legal(
    shared: Shared,
    rules: web::Data<Arc<StandardChess>>,
    web::Path((px, py, wx, wy, zoom)): web::Path<(String, String, String, String, String)>,
) -> Result<HttpResponse, Error> {
    let (board, cvar) = &**shared;
    let bigpx = BigInt::from_str(&px).map_err(|_| Error::new())?;
    let bigpy = BigInt::from_str(&py).map_err(|_| Error::new())?;
    let bigwx = BigInt::from_str(&wx).map_err(|_| Error::new())?;
    let bigwy = BigInt::from_str(&wy).map_err(|_| Error::new())?;
    let bigzoom = BigInt::from_str(&zoom).map_err(|_| Error::new())?;

    let mut b = board.lock().await;

    let mut xx = bigwx.clone();
    let wwx = bigwx + bigzoom.clone();
    let wwy = bigwy.clone() + bigzoom;
    let mut results = Vec::new();
    while xx < wwx {
        let mut yy = bigwy.clone();
        while yy < wwy {
            if Board::is_move_legal(&mut b, &rules, &bigpx, &bigpy, &xx, &yy) {
                results.push(format!("[{}, {}]", xx, yy));
            }
            yy += 1;
        }
        xx += 1;
    }
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("[{}]", results.join(","))))
}

#[get("/move/{px}/{py}/{dx}/{dy}")]
pub async fn get_move(
    shared: Shared,
    rules: web::Data<Arc<StandardChess>>,
    web::Path((px, py, dx, dy)): web::Path<(String, String, String, String)>,
) -> Result<HttpResponse, Error> {
    let (board, cvar) = &**shared;
    let bigpx = BigInt::from_str(&px).map_err(|_| Error::new())?;
    let bigpy = BigInt::from_str(&py).map_err(|_| Error::new())?;
    let bigdx = BigInt::from_str(&dx).map_err(|_| Error::new())?;
    let bigdy = BigInt::from_str(&dy).map_err(|_| Error::new())?;

    let mut b = board.lock().await;

    if let Some(m) = Board::move_legal(&mut b, &rules, &bigpx, &bigpy, &bigdx, &bigdy) {
        b.do_move(m);
        b.turn += 1;
    }
    cvar.notify_all();
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("swag"))
}


#[get("/promote/{px}/{py}/{p}")]
pub async fn get_promote(
    shared: Shared,
    rules: web::Data<Arc<StandardChess>>,
    web::Path((px, py, p)): web::Path<(String, String, String)>,
) -> Result<HttpResponse, Error> {
    let (board, cvar) = &**shared;
    let bigpx = BigInt::from_str(&px).map_err(|_| Error::new())?;
    let bigpy = BigInt::from_str(&py).map_err(|_| Error::new())?;

    let mut b = board.lock().await;

    b.promote(&bigpx, &bigpy, p);
    cvar.notify_all();
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("swag"))
}