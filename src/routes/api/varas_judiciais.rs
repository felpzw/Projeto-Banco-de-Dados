use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::StatusCode;
use tuono_lib::Request;
use tuono_app::{connect_db};

use serde_json::{json, Value};

#[tuono_lib::api(GET)]
pub async fn vara_judicial(_req: Request) -> Json<Value> {
    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Json(json!({ "error": format!("Database connection error: {}", e) }));
        }
    };

    let rows = match client_db
        .query(
            "SELECT id_vara_judicial, nome_vara FROM Vara_Judicial ORDER BY nome_vara ASC;",
            &[],
        )
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch varas_judiciais: {}", e);
            return Json(json!({ "error": format!("Failed to fetch varas_judiciais: {}", e) }));
        }
    };

    let varas_list: Vec<Value> = rows.into_iter().map(|row| {
        json!({
            "id": row.get::<_, i32>("id_vara_judicial"), // Usar "id"
            "nome": row.get::<_, String>("nome_vara"),   // Usar "nome"
        })
    }).collect();

    Json(json!(varas_list))
}