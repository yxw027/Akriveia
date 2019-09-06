
use common::*;
use futures::{ Stream, Future, IntoFuture, };
use na;
use tokio_postgres::row::Row;
use tokio_postgres::types::Type;

pub fn row_to_map(row: &Row) -> Map {
    let mut entry = Map::new();
    for (i, column) in row.columns().iter().enumerate() {
        match column.name() {
            "m_id" => entry.id = row.get(i),
            "m_blueprint" => {}, //panic!("blueprint data not handled yet"),
            "m_bounds" => {
                let bounds: Vec<f64> = row.get(i);
                entry.bounds = na::Vector2::new(bounds[0], bounds[1]);
            }
            "m_scale" => entry.scale = row.get(i),
            "m_name" => entry.name = row.get(i),
            "m_note" => entry.note = row.get(i),
            unhandled if unhandled.starts_with("m_") => { panic!("unhandled beacon column {}", unhandled); },
            _ => {},
        }
    }
    entry
}

pub fn select_maps(mut client: tokio_postgres::Client) -> impl Future<Item=(tokio_postgres::Client, Vec<Map>), Error=tokio_postgres::Error> {
    // TODO paging
    client
        .prepare("
            SELECT * FROM runtime.maps
        ")
        .and_then(move |statement| {
            client
                .query(&statement, &[])
                .collect()
                .into_future()
                .map(|rows| {
                    (client, rows.into_iter().map(|row| row_to_map(&row)).collect())
                })
        })
}

pub fn select_map(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<Map>), Error=tokio_postgres::Error> {
    client
        .prepare("
            SELECT * FROM runtime.maps
            WHERE m_id = $1::INTEGER
        ")
        .and_then(move |statement| {
            client
                .query(&statement, &[&id])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_map(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn insert_map(mut client: tokio_postgres::Client, map: Map) -> impl Future<Item=(tokio_postgres::Client, Option<Map>), Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            INSERT INTO runtime.maps (
                m_bounds,
                m_name,
                m_note,
                m_scale,
            )
            VALUES( $1, $2, $3, $4 )
            RETURNING *
        ", &[
            Type::FLOAT8_ARRAY,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::FLOAT8,
        ])
        .and_then(move |statement| {
            let bounds = vec![map.bounds[0], map.bounds[1]];
            client
                .query(&statement, &[
                    &bounds,
                    &map.name,
                    &map.note,
                    &map.scale,
                ])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_map(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn update_map(mut client: tokio_postgres::Client, map: Map) -> impl Future<Item=(tokio_postgres::Client, Option<Map>), Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            UPDATE runtime.maps
            SET
                m_bounds = $1,
                m_name = $2,
                m_note = $3,
                m_scale = $4,
             WHERE
                m_id = $5
            RETURNING *
        ", &[
            Type::FLOAT8_ARRAY,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::FLOAT8,
            Type::INT4,
        ])
        .and_then(move |statement| {
            let bounds = vec![map.bounds[0], map.bounds[1]];
            client
                .query(&statement, &[
                    &bounds,
                    &map.name,
                    &map.note,
                    &map.scale,
                    &map.id,
                ])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_map(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn delete_map(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            DELETE FROM runtime.maps
            WHERE (
                m_id = $1
            )
        ", &[
            Type::INT4,
        ])
        .and_then(move |statement| {
            client
                .query(&statement, &[&id])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(_row, _next)| {
                    client
                })
        })
}
