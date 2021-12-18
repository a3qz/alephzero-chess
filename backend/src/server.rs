use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use actix_web::{web, App, HttpServer};

use actix_web::{get, put, HttpResponse};
use num_bigint::BigInt;
use crate::piece;
use crate::piece::Piece;
use rusqlite::Connection;
use crate::piece_rules::StandardChess;


use crate::board::Board;
use crate::board_serializer::board_serialize;
use crate::error::*;
use actix_files as fs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut b = Board::new();
    b.place_piece(Piece::new("rook".to_string(), piece::Color::Black, 0.into(), 0.into()));
    b.place_piece(Piece::new("rook".to_string(), piece::Color::White, 7.into(), 0.into()));
    b.place_piece(Piece::new("rook".to_string(), piece::Color::Black, 0.into(), 7.into()));
    b.place_piece(Piece::new("rook".to_string(), piece::Color::White, 7.into(), 7.into()));
    b.place_piece(Piece::new("knight".to_string(), piece::Color::Black, 0.into(), 1.into()));
    b.place_piece(Piece::new("knight".to_string(), piece::Color::White, 7.into(), 1.into()));
    b.place_piece(Piece::new("knight".to_string(), piece::Color::Black, 0.into(), 6.into()));
    b.place_piece(Piece::new("knight".to_string(), piece::Color::White, 7.into(), 6.into()));
    b.place_piece(Piece::new("bishop".to_string(), piece::Color::Black, 0.into(), 2.into()));
    b.place_piece(Piece::new("bishop".to_string(), piece::Color::White, 7.into(), 2.into()));
    b.place_piece(Piece::new("bishop".to_string(), piece::Color::Black, 0.into(), 5.into()));
    b.place_piece(Piece::new("bishop".to_string(), piece::Color::White, 7.into(), 5.into()));
    b.place_piece(Piece::new("king".to_string(), piece::Color::Black, 0.into(), 4.into()));
    b.place_piece(Piece::new("king".to_string(), piece::Color::White, 7.into(), 4.into()));
    b.place_piece(Piece::new("queen".to_string(), piece::Color::Black, 0.into(), 3.into()));
    b.place_piece(Piece::new("queen".to_string(), piece::Color::White, 7.into(), 3.into()));
    
    let board = web::Data::new(Arc::new(Mutex::new(b)));
    HttpServer::new(move || App::new().service(get).service(get_legal).service(get_move).app_data(board.clone()).data(Arc::new(StandardChess::new())).service(fs::Files::new("/", "./static").index_file("index.html")))
        .bind("127.0.0.1:8080")?
        .run()
        .await?;
    Ok(())
}


#[get("/board")]
pub async fn get(
    board: web::Data<Arc<Mutex<Board>>>,
) -> Result<HttpResponse, Error> {
    let b = board.lock().unwrap();
    Ok(HttpResponse::Ok().content_type("application/json").body(board_serialize(&b)))
}

#[get("/legal/{px}/{py}/{wx}/{wy}/{zoom}")]
pub async fn get_legal(
    board: web::Data<Arc<Mutex<Board>>>,
    rules: web::Data<Arc<StandardChess>>,
    web::Path((px, py, wx, wy, zoom)): web::Path<(String, String, String, String, String)>
) -> Result<HttpResponse, Error> {
    let bigpx = BigInt::from_str(&px).map_err(|_| Error::new())?;
    let bigpy = BigInt::from_str(&py).map_err(|_| Error::new())?;
    let bigwx = BigInt::from_str(&wx).map_err(|_| Error::new())?;
    let bigwy = BigInt::from_str(&wy).map_err(|_| Error::new())?;
    let bigzoom = BigInt::from_str(&zoom).map_err(|_| Error::new())?;

    let mut b = board.lock().unwrap();
    
    let mut xx = bigwx.clone();
    let wwx = bigwx + bigzoom.clone();
    let wwy = bigwy.clone() + bigzoom;
    let mut results = Vec::new();
    while xx <  wwx {
        let mut yy = bigwy.clone();
        while yy < wwy {
            if Board::is_move_legal(&mut b, &rules, &bigpx, &bigpy, &xx, &yy) {
                results.push(format!("[{}, {}]", xx, yy));
            }
            yy += 1;
        }
        xx += 1;
    }
    Ok(HttpResponse::Ok().content_type("application/json").body(format!("[{}]", results.join(","))))
}

#[get("/move/{px}/{py}/{dx}/{dy}")]
pub async fn get_move(
    board: web::Data<Arc<Mutex<Board>>>,
    rules: web::Data<Arc<StandardChess>>,
    web::Path((px, py, dx, dy)): web::Path<(String, String, String, String)>
) -> Result<HttpResponse, Error> {
    let bigpx = BigInt::from_str(&px).map_err(|_| Error::new())?;
    let bigpy = BigInt::from_str(&py).map_err(|_| Error::new())?;
    let bigdx = BigInt::from_str(&dx).map_err(|_| Error::new())?;
    let bigdy = BigInt::from_str(&dy).map_err(|_| Error::new())?;

    let mut b = board.lock().unwrap();
   
    if Board::is_move_legal(&mut b, &rules, &bigpx, &bigpy, &bigdx, &bigdy) {
        b.do_move(&bigpx, &bigpy, &bigdx, &bigdy);
    }
   
    Ok(HttpResponse::Ok().content_type("application/json").body("swag"))
}