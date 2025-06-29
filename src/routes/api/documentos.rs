use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::{StatusCode, HeaderMap, HeaderValue, header};
use tuono_lib::Request;
use tuono_app::{connect_db, extract_query_values};

use serde_json::{json, Value};
use chrono::NaiveDate;
use mime_guess;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

// --- Structs para Payload JSON (Frontend -> Backend) ---
#[derive(Debug, Serialize, Deserialize)]
struct DocumentPayload {
    id_caso: i32,
    descricao: String,
    data_envio: String, // String no formato AAAA-MM-DD
    nome_arquivo: String,
    arquivo_base64: String, // Conteúdo Base64 do arquivo
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentUpdatePayload {
    id: i32, // ID do documento a ser atualizado
    id_caso: i32,
    descricao: String,
    data_envio: String, // String no formato AAAA-MM-DD
    nome_arquivo: String,
    arquivo_base64: Option<String>, // Conteúdo Base64 do arquivo (opcional para atualização)
}


// GET /api/documentos (Listar documentos ou buscar específico, com opção de download)
#[tuono_lib::api(GET)]
async fn documento(_req: Request) -> impl IntoResponse {
    let query_string = _req.uri.query().unwrap_or("");
    let query_values_result = tuono_app::extract_query_values(query_string);

    let client_db = match tuono_app::connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)}))).into_response();
        }
    };

    // Lógica para download direto do arquivo
    if let Ok(values) = &query_values_result {
        if let (Some(id_str), Some(download_str)) = (values.get("id"), values.get("download")) {
            if download_str == "true" {
                let id = match id_str.parse::<i32>() {
                    Ok(id) => id,
                    Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID do documento deve ser um número inteiro."}))).into_response(),
                };

                let row = match client_db
                    .query_opt("SELECT nome_arquivo, arquivo FROM Documento WHERE id_documento = $1;", &[&id])
                    .await
                {
                    Ok(row) => row,
                    Err(e) => {
                        eprintln!("Failed to fetch document content: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to fetch document content: {}", e)}))).into_response();
                    }
                };

                if let Some(r) = row {
                    let nome_arquivo: String = r.get("nome_arquivo");
                    let arquivo_bytes: Option<Vec<u8>> = r.get("arquivo");

                    if let Some(bytes) = arquivo_bytes {
                        let mime_type = mime_guess::from_path(&nome_arquivo)
                            .first_or_octet_stream()
                            .to_string();

                        let mut headers = HeaderMap::new();
                        headers.insert(header::CONTENT_TYPE, HeaderValue::from_str(&mime_type).unwrap());
                        headers.insert(
                            header::CONTENT_DISPOSITION,
                            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", nome_arquivo)).unwrap(),
                        );
                        return (StatusCode::OK, headers, bytes).into_response();
                    } else {
                        return (StatusCode::NOT_FOUND, Json(json!({"error": "Conteúdo do arquivo não encontrado para este documento."}))).into_response();
                    }
                } else {
                    return (StatusCode::NOT_FOUND, Json(json!({"error": "Documento não encontrado."}))).into_response();
                }
            }
        }
    }

    // Lógica para listar todos os documentos ou buscar metadados de um específico
    let mut is_specific_id_requested = false; // Flag para rastrear se um ID foi solicitado
    let rows_result = if let Ok(values) = query_values_result {
        if let Some(id_str) = values.get("id").cloned() {
            is_specific_id_requested = true;
            let id = match id_str.parse::<i32>() {
                Ok(id) => id,
                Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID parameter must be an integer."}))).into_response(),
            };
            match client_db
                .query(
                    "SELECT id_documento, id_caso, descricao, data_envio, nome_arquivo FROM Documento WHERE id_documento = $1;",
                    &[&id],
                )
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to fetch specific document: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to fetch specific document: {}", e)}))).into_response();
                }
            }
        } else {
            // Nenhum ID na query, retorna todos
            match client_db
                .query(
                    "SELECT id_documento, id_caso, descricao, data_envio, nome_arquivo FROM Documento ORDER BY data_envio DESC;",
                    &[],
                )
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to fetch all documents: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to fetch documents: {}", e)}))).into_response();
                }
            }
        }
    } else { // Erro ao extrair query values, mas não é um download, então lista tudo
        match client_db
            .query(
                "SELECT id_documento, id_caso, descricao, data_envio, nome_arquivo FROM Documento ORDER BY data_envio DESC;",
                &[],
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to fetch all documents: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to fetch documents: {}", e)}))).into_response();
            }
        }
    };


    let documents: Vec<Value> = rows_result.into_iter().map(|row| { // Usar rows_result aqui
        let data_envio: Option<NaiveDate> = row.get("data_envio");
        json!({
            "id_documento": row.get::<_, i32>("id_documento"),
            "id_caso": row.get::<_, i32>("id_caso"),
            "descricao": row.get::<_, String>("descricao"),
            "data_envio": data_envio.map(|d| d.to_string()),
            "nome_arquivo": row.get::<_, String>("nome_arquivo"),
        })
    }).collect();

    // Se um ID específico foi solicitado E (&&) não há documentos encontrados
    if is_specific_id_requested {
        if documents.is_empty() {
            return (StatusCode::NOT_FOUND, Json(json!({"error": "Documento não encontrado."}))).into_response();
        } else {
            // Se um ID específico foi solicitado E documentos foram encontrados,
            // retorne o primeiro (e único) objeto diretamente, não um array.
            return (StatusCode::OK, Json(documents[0].clone())).into_response();
        }
    }

    // Caso contrário, retorne a lista completa de documentos
    Json(json!(documents)).into_response()
}

// POST /api/documentos (Adicionar novo documento com Base64 no corpo JSON)
#[tuono_lib::api(POST)]
async fn create_documento(_req: Request) -> impl IntoResponse {
    let payload: DocumentPayload = match _req.body() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to read/parse JSON body from Request: {:?}", e);
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid request body or JSON parsing error: {:?}", e)})));
        }
    };

    let arquivo_bytes = match general_purpose::STANDARD.decode(&payload.arquivo_base64) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to decode Base64 content: {}", e);
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Conteúdo Base64 inválido: {}", e)})));
        }
    };

    let id_caso = payload.id_caso;
    let descricao = payload.descricao;
    let data_envio = match NaiveDate::parse_from_str(&payload.data_envio, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Data de envio inválida. Use o formato AAAA-MM-DD."}))),
    };
    let nome_arquivo = payload.nome_arquivo;

    let client_db = match tuono_app::connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)})));
        }
    };

    let caso_exists = client_db.query_opt("SELECT 1 FROM Caso WHERE id_caso = $1;", &[&id_caso]).await;
    if let Err(e) = caso_exists {
        eprintln!("Failed to check Caso existence: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database error checking Case ID: {}", e)})));
    }
    if caso_exists.unwrap().is_none() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID do caso (id_caso) não existe."})));
    }

    let insert_documento_query = "INSERT INTO Documento (id_caso, descricao, data_envio, nome_arquivo, arquivo) VALUES ($1, $2, $3, $4, $5) RETURNING id_documento;";
    let rows = match client_db.query(insert_documento_query, &[&id_caso, &descricao, &data_envio, &nome_arquivo, &arquivo_bytes]).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to insert Documento: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert document: {}", e)})));
        }
    };

    let id_documento: i32 = rows[0].get("id_documento");

    (StatusCode::CREATED, Json(json!({"message": "Documento adicionado com sucesso", "id_documento": id_documento})))
}

// PUT /api/documentos (Atualizar documento com Base64 no corpo JSON, ou não enviar arquivo para manter o existente)
#[tuono_lib::api(PUT)]
async fn update_documento(_req: Request) -> impl IntoResponse {
    let payload: DocumentUpdatePayload = match _req.body() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to read/parse JSON body from Request: {:?}", e);
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid request body or JSON parsing error: {:?}", e)})));
        }
    };

    let id_documento = payload.id;
    let id_caso = payload.id_caso;
    let descricao = payload.descricao;
    let data_envio = match NaiveDate::parse_from_str(&payload.data_envio, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Data de envio inválida. Use o formato AAAA-MM-DD."}))),
    };
    let nome_arquivo = payload.nome_arquivo;

    let owned_bytes: Vec<u8>;
    let arquivo_bytes_ref: Option<&Vec<u8>> = if let Some(base64_str) = payload.arquivo_base64 {
        match general_purpose::STANDARD.decode(base64_str) {
            Ok(bytes) => {
                owned_bytes = bytes;
                Some(&owned_bytes)
            },
            Err(e) => {
                eprintln!("Failed to decode Base64 content for update: {}", e);
                return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Conteúdo Base64 inválido para atualização: {}", e)})));
            }
        }
    } else {
        None
    };

    let client_db = match tuono_app::connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)})));
        }
    };

    let caso_exists = client_db.query_opt("SELECT 1 FROM Caso WHERE id_caso = $1;", &[&id_caso]).await;
    if let Err(e) = caso_exists {
        eprintln!("Failed to check Caso existence: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database error checking Case ID: {}", e)})));
    }
    if caso_exists.unwrap().is_none() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID do caso (id_caso) não existe."})));
    }

    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    
    params.push(&id_caso);
    params.push(&descricao);
    params.push(&data_envio);
    params.push(&nome_arquivo);

    let update_query;

    if let Some(bytes_ref) = arquivo_bytes_ref {
        params.push(bytes_ref);
        params.push(&id_documento);
        update_query = "UPDATE Documento SET id_caso = $1, descricao = $2, data_envio = $3, nome_arquivo = $4, arquivo = $5 WHERE id_documento = $6;";
    } else {
        params.push(&id_documento);
        update_query = "UPDATE Documento SET id_caso = $1, descricao = $2, data_envio = $3, nome_arquivo = $4 WHERE id_documento = $5;";
    }
    
    match client_db.execute(update_query, &params).await {
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
    let query_values = match tuono_app::extract_query_values(query_string) {
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

    let client_db = match tuono_app::connect_db().await {
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