use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::StatusCode;
use tuono_lib::Request;
use tuono_app::{connect_db};

use serde_json::{json, Value};

#[tuono_lib::api(GET)]
pub async fn status(_req: Request) -> Json<Value> {
    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Json(json!({ "error": format!("Database connection error: {}", e) }));
        }
    };

    let rows = match client_db
        .query(
            "SELECT id_status, descricao FROM Status ORDER BY descricao ASC;",
            &[],
        )
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch statuses: {}", e);
            return Json(json!({ "error": format!("Failed to fetch statuses: {}", e) }));
        }
    };

    let status_list: Vec<Value> = rows.into_iter().map(|row| {
        json!({
            "id": row.get::<_, i32>("id_status"), // Retorna id como número
            "nome": row.get::<_, String>("descricao"),
        })
    }).collect();

    Json(json!(status_list))
}