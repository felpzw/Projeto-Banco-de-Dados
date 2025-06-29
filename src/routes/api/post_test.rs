use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::StatusCode;
use tuono_lib::Request;
use tuono_app::connect_db; // Importa connect_db do tuono_app

use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use pdf_extract::{extract_text_from_mem, OutputError}; 

#[derive(Debug, Deserialize)]
struct DocumentIdPayload {
    id_documento: i32,
}

#[tuono_lib::api(POST)]
async fn post_test(_req: Request) -> impl IntoResponse {

    let payload: DocumentIdPayload = match _req.body() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to read/parse JSON body from Request: {:?}", e);
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid request body or JSON parsing error: {:?}", e)})));
        }
    };

    let id_documento_to_extract = payload.id_documento;

    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)})));
        }
    };

    let row = match client_db
        .query_opt("SELECT arquivo FROM Documento WHERE id_documento = $1;", &[&id_documento_to_extract])
        .await
    {
        Ok(row) => row,
        Err(e) => {
            eprintln!("Failed to fetch document from DB: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to fetch document content from DB: {}", e)})));
        }
    };

    let document_bytes: Vec<u8> = if let Some(r) = row {
        match r.get("arquivo") {
            Some(bytes) => bytes,
            None => return (StatusCode::NOT_FOUND, Json(json!({"error": "Conteúdo do arquivo não encontrado no documento especificado."}))),
        }
    } else {
        return (StatusCode::NOT_FOUND, Json(json!({"error": "Documento não encontrado com o ID fornecido."})));
    };

    
    let extracted_text = match extract_text_from_mem(&document_bytes) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Failed to extract text from PDF: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to extract text from PDF: {:?}", e)})));
        }
    };

    (StatusCode::OK, Json(json!({
        "message": "Texto extraído com sucesso",
        "document_id": id_documento_to_extract,
        "extracted_text": extracted_text
    })))
}