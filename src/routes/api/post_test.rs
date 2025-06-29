use serde::{Deserialize, Serialize};
use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::StatusCode;
use tuono_lib::Request;
use tuono_lib::tokio;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};

#[derive(Deserialize)]
struct PdfUpload {
    filename: String,
    content_base64: String, // PDF em Base64
    metadata: Option<Value> // Campos adicionais
}

#[tuono_lib::api(POST)]
async fn upload_pdf(req: Request) -> impl IntoResponse {
    let body: PdfUpload = match req.body() {
        Ok(body) => body,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, 
                Json(json!({"error": "Invalid request", "details": format!("{:?}", e)}))) // Alterado aqui
        }
    };

    // Decodifica o Base64
    let pdf_bytes = match general_purpose::STANDARD.decode(&body.content_base64) {
        Ok(bytes) => bytes,
        Err(e) => {
            return (StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid Base64", "details": e.to_string()})))
        }
    };

    // Valida se é um PDF (opcional)
    if !pdf_bytes.starts_with(b"%PDF-") {
        return (StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid PDF file"})));
    }

    // Salva o arquivo (exemplo)
    if let Err(e) = tokio::fs::write(&body.filename, &pdf_bytes).await {
        return (StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to save file", "details": e.to_string()})));
    }

    (StatusCode::OK, Json(json!({
        "message": "PDF received successfully",
        "filename": body.filename,
        "size_bytes": pdf_bytes.len()
    })))
}