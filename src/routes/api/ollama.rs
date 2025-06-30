use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::StatusCode;
use tuono_lib::Request;
use tuono_app::connect_db;
use pdf_extract::extract_text_from_mem;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInternal>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelInternal {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct OllamaGenerateRequest {
    file_name: String, 
    question: String,  
    model: String,     
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OllamaRawGenerateResponse {
    response: String,
}


#[tuono_lib::api(POST)]
pub async fn ollama_post_generate(_req: Request, fetch: reqwest::Client) -> impl IntoResponse {
    let payload: OllamaGenerateRequest = match _req.body() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to parse request body: {:?}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("Invalid request body: {:?}", e)}))
            );
        }
    };

    let file_name = payload.file_name;
    let user_question = payload.question;
    let ollama_model = payload.model;

    // 2. Conectar ao banco de dados
    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database connection error: {}", e)}))
            );
        }
    };

    let row = match client_db
        .query_opt("SELECT arquivo FROM Documento WHERE nome_arquivo = $1;", &[&file_name])
        .await
    {
        Ok(row) => row,
        Err(e) => {
            eprintln!("Failed to fetch document from DB: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to fetch document content from DB: {}", e)}))
            );
        }
    };

    let document_bytes: Vec<u8> = if let Some(r) = row {
        match r.get("arquivo") {
            Some(bytes) => bytes,
            None => return (StatusCode::NOT_FOUND, Json(json!({"error": "Conteúdo do arquivo não encontrado para o nome fornecido."}))),
        }
    } else {
        return (StatusCode::NOT_FOUND, Json(json!({"error": format!("Documento com nome '{}' não encontrado.", file_name)})));
    };

    let extracted_text = match extract_text_from_mem(&document_bytes) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Failed to extract text from PDF: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to extract text from PDF: {:?}", e)}))
            );
        }
    };

    let full_prompt = format!(
        "Com base no seguinte documento, responda à pergunta do usuário. Se a informação não estiver no documento, diga que não pode responder.\n\nDocumento:\n```\n{}\n```\n\nPergunta do Usuário: {}",
        extracted_text, user_question
    );

    let ollama_env = std::env::var("OLLAMA_API_URL").expect("Fail to read OLLAMA_API_URL env");
    let ollama_api_url: &str = &format!("{}api/generate", ollama_env);

    let ollama_request_body = json!({
        "model": ollama_model,
        "prompt": full_prompt,
        "stream": false
    });

    let ollama_response_raw = match fetch.post(ollama_api_url)
        .json(&ollama_request_body)
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to send request to Ollama API (generate): {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to connect to Ollama API for generation: {}", e)}))
            );
        }
    };

    if !ollama_response_raw.status().is_success() {
        let status = ollama_response_raw.status();
        let text = ollama_response_raw.text().await.unwrap_or_else(|_| "No response body".to_string());
        eprintln!("Ollama API returned an error during generation: Status={}, Body={}", status, text);
        return (
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": format!("Ollama API generation error: Status {}, Body: {}", status, text)}))
        );
    }

    let ollama_response_data: OllamaRawGenerateResponse = match ollama_response_raw.json().await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to parse Ollama API generation response: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to parse Ollama API generation response: {}", e)}))
            );
        }
    };

    // 7. Retornar a resposta do Ollama para o frontend
    (StatusCode::OK, Json(json!({
        "message": "Resposta do LLM obtida com sucesso",
        "llm_response": ollama_response_data.response
    })))
}