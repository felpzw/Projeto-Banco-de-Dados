use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::StatusCode;
use tuono_lib::Request;
use tuono_app::{connect_db};

use serde_json::{json, Value};

#[tuono_lib::api(GET)]
pub async fn advogado(_req: Request) -> Json<Value> {
    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Json(json!({ "error": format!("Database connection error: {}", e) }));
        }
    };

    let rows = match client_db
        .query(
            "SELECT id_advogado, nome, oab FROM Advogado ORDER BY nome ASC;",
            &[],
        )
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch lawyers: {}", e);
            return Json(json!({ "error": format!("Failed to fetch lawyers: {}", e) }));
        }
    };

    let lawyers_list: Vec<Value> = rows.into_iter().map(|row| {
        json!({
            "id": row.get::<_, i32>("id_advogado"),
            "nome": row.get::<_, String>("nome"),
            "oab": row.get::<_, String>("oab"),
        })
    }).collect();

    Json(json!(lawyers_list))
}