use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::{StatusCode};
use tuono_lib::Request;
use tuono_app::{connect_db, extract_query_values};

use serde_json::{json, Value};
use chrono::NaiveDate;

// GET /api/documentos (Listar todos os documentos ou um específico por ID de documento)
#[tuono_lib::api(GET)]
async fn documento(_req: Request) -> Json<Value> {
    let query_string = _req.uri.query().unwrap_or("");
    let query_values = match extract_query_values(query_string) {
        Ok(values) => values,
        Err(e) => {
            // Se a query string estiver vazia, retorna todos os documentos (comportamento de lista)
            if query_string.is_empty() {
                let client_db = match connect_db().await {
                    Ok(client) => client,
                    Err(e) => {
                        eprintln!("Failed to connect to database: {}", e);
                        return Json(json!({"error": format!("Database connection error: {}", e)}));
                    }
                };

                let rows = match client_db
                    .query(
                        "SELECT id_documento, id_caso, descricao, data_envio, tipo, nome_arquivo FROM Documento ORDER BY data_envio DESC;",
                        &[],
                    )
                    .await
                {
                    Ok(rows) => rows,
                    Err(e) => {
                        eprintln!("Failed to fetch documents: {}", e);
                        return Json(json!({"error": format!("Failed to fetch documents: {}", e)}));
                    }
                };

                let documents: Vec<Value> = rows.into_iter().map(|row| {
                    let data_envio: Option<NaiveDate> = row.get("data_envio");
                    json!({
                        "id_documento": row.get::<_, i32>("id_documento"),
                        "id_caso": row.get::<_, i32>("id_caso"),
                        "descricao": row.get::<_, String>("descricao"),
                        "data_envio": data_envio.map(|d| d.to_string()),
                        "tipo": row.get::<_, String>("tipo"),
                        "nome_arquivo": row.get::<_, String>("nome_arquivo"),
                    })
                }).collect();
                return Json(json!(documents));
            } else {
                // Se a query string não está vazia, mas tem erro de parsing, é um BAD_REQUEST
                eprintln!("Failed to extract query values for GET: {}", e);
                return Json(json!({"error": format!("Invalid query parameters: {}", e)}));
            }
        }
    };

    // Se há ID na query, busca um documento específico
    let id_str = match query_values.get("id") {
        Some(id) => id,
        _ => return Json(json!({"error": "ID parameter is required to fetch a specific document."})),
    };

    let id = match id_str.parse::<i32>() {
        Ok(id) => id,
        Err(_) => return Json(json!({"error": "ID parameter must be an integer."})),
    };

    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Json(json!({"error": format!("Database connection error: {}", e)}));
        }
    };

    let rows = match client_db
        .query(
            "SELECT id_documento, id_caso, descricao, data_envio, tipo, nome_arquivo FROM Documento WHERE id_documento = $1;",
            &[&id],
        )
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to execute query: {}", e);
            return Json(json!({"error": format!("Failed to fetch document: {}", e)}));
        }
    };

    if rows.is_empty() {
        return Json(json!({"error": "Document not found."}));
    }

    let row = &rows[0];
    let data_envio: Option<NaiveDate> = row.get("data_envio");
    Json(
        json!({
            "id_documento": row.get::<_, i32>("id_documento"),
            "id_caso": row.get::<_, i32>("id_caso"),
            "descricao": row.get::<_, String>("descricao"),
            "data_envio": data_envio.map(|d| d.to_string()),
            "tipo": row.get::<_, String>("tipo"),
            "nome_arquivo": row.get::<_, String>("nome_arquivo"),
        })
    )
}

// POST /api/documentos (Adicionar novo documento)
#[tuono_lib::api(POST)]
async fn create_documento(_req: Request) -> impl IntoResponse {
    let query_string = _req.uri.query().unwrap_or("");
    println!("Query Recebida para POST (Documento): {}", query_string);
    let query_values = match extract_query_values(query_string) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to extract query values for POST (Documento): {}", e);
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid query parameters: {}", e)})));
        }
    };

    let id_caso_str = match query_values.get("id_caso") {
        Some(id) => id,
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "id_caso is required."}))),
    };
    let id_caso = match id_caso_str.parse::<i32>() {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "id_caso must be an integer."}))),
    };

    let descricao = match query_values.get("descricao") {
        Some(d) => d.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Descricao is required."}))),
    };

    let data_envio = query_values.get("data_envio").and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    
    let tipo = match query_values.get("tipo") {
        Some(t) => t.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Tipo is required."}))),
    };
    let nome_arquivo = match query_values.get("nome_arquivo") {
        Some(n) => n.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Nome do arquivo is required."}))),
    };

    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)})));
        }
    };

    // Verify if id_caso exists in the Caso table
    let caso_exists = client_db.query_opt("SELECT 1 FROM Caso WHERE id_caso = $1;", &[&id_caso]).await;
    if let Err(e) = caso_exists {
        eprintln!("Failed to check Caso existence: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database error checking Case ID: {}", e)})));
    }
    if caso_exists.unwrap().is_none() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Case ID (id_caso) does not exist."})));
    }


    let insert_documento_query = "INSERT INTO Documento (id_caso, descricao, data_envio, tipo, nome_arquivo) VALUES ($1, $2, $3, $4, $5) RETURNING id_documento;";
    let rows = match client_db.query(insert_documento_query, &[&id_caso, &descricao, &data_envio, &tipo, &nome_arquivo]).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to insert Documento: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert document: {}", e)})));
        }
    };

    let id_documento: i32 = rows[0].get("id_documento");

    (StatusCode::CREATED, Json(json!({"message": "Documento adicionado com sucesso", "id_documento": id_documento})))
}

// PUT /api/documentos (Atualizar documento)
#[tuono_lib::api(PUT)]
async fn update_documento(_req: Request) -> impl IntoResponse {
    let query_string = _req.uri.query().unwrap_or("");
    println!("Query Recebida para PUT (Documento): {}", query_string);
    let query_values = match extract_query_values(query_string) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to extract query values for PUT (Documento): {}", e);
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid query parameters: {}", e)})));
        }
    };

    let id_documento_str = match query_values.get("id") { // Usar "id" para consistência com o frontend
        Some(id) => id,
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID do documento é obrigatório."}))),
    };
    let id_documento = match id_documento_str.parse::<i32>() {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID do documento deve ser um número inteiro."}))),
    };

    let id_caso_str = match query_values.get("id_caso") {
        Some(id) => id,
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "id_caso é obrigatório."}))),
    };
    let id_caso = match id_caso_str.parse::<i32>() {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "id_caso deve ser um número inteiro."}))),
    };

    let descricao = match query_values.get("descricao") {
        Some(d) => d.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Descricao é obrigatória."}))),
    };

    let data_envio = query_values.get("data_envio").and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let tipo = match query_values.get("tipo") {
        Some(t) => t.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Tipo é obrigatório."}))),
    };
    let nome_arquivo = match query_values.get("nome_arquivo") {
        Some(n) => n.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Nome do arquivo é obrigatório."}))),
    };

    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)})));
        }
    };

    // Verify if id_caso exists in the Caso table
    let caso_exists = client_db.query_opt("SELECT 1 FROM Caso WHERE id_caso = $1;", &[&id_caso]).await;
    if let Err(e) = caso_exists {
        eprintln!("Failed to check Caso existence: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database error checking Case ID: {}", e)})));
    }
    if caso_exists.unwrap().is_none() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Case ID (id_caso) does not exist."})));
    }


    let update_documento_query = "UPDATE Documento SET id_caso = $1, descricao = $2, data_envio = $3, tipo = $4, nome_arquivo = $5 WHERE id_documento = $6;";
    match client_db.execute(update_documento_query, &[&id_caso, &descricao, &data_envio, &tipo, &nome_arquivo, &id_documento]).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                (StatusCode::OK, Json(json!({"message": "Documento atualizado com sucesso"})))
            } else {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Documento não encontrado."})))
            }
        },
        Err(e) => {
            eprintln!("Failed to update Documento: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update document: {}", e)})))
        }
    }
}

// DELETE /api/documentos (Excluir documento)
#[tuono_lib::api(DELETE)]
async fn delete_documento(_req: Request) -> impl IntoResponse {
    let query_string = _req.uri.query().unwrap_or("");
    println!("Query Recebida para DELETE (Documento): {}", query_string);
    let query_values = match extract_query_values(query_string) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to extract query values for DELETE (Documento): {}", e);
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid query parameters: {}", e)})));
        }
    };

    let id_documento_str = match query_values.get("id") {
        Some(id) => id,
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID do documento é obrigatório."}))),
    };
    let id_documento = match id_documento_str.parse::<i32>() {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID do documento deve ser um número inteiro."}))),
    };

    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)})));
        }
    };

    let delete_documento_query = "DELETE FROM Documento WHERE id_documento = $1;";
    match client_db.execute(delete_documento_query, &[&id_documento]).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                (StatusCode::OK, Json(json!({"message": "Documento excluído com sucesso."})))
            } else {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Documento não encontrado."})))
            }
        },
        Err(e) => {
            eprintln!("Failed to delete Documento: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to delete document: {}", e)})))
        }
    }
}