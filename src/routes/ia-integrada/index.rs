// src/routes/ia-integrada/index.rs
use serde::{Deserialize, Serialize};
use tuono_lib::{Props, Request, Response};
use tuono_lib::axum::http;
use tuono_app::connect_db;

// --- Structs para dados de modelos Ollama (para o frontend) ---
#[derive(Debug, Serialize, Deserialize, Clone)]
struct OllamaModelFrontend {
    id: String,
    nome: String,
}

// --- Structs para a Resposta da API do Ollama /api/tags (interna para o proxy) ---
#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInternal>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelInternal {
    name: String,
}


// --- Structs para documentos (para o frontend) ---
#[derive(Debug, Serialize, Deserialize, Clone)]
struct DocumentFrontend {
    id_documento: i32,
    nome_arquivo: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct IaIntegratedPageProps {
    ollama_models: Vec<OllamaModelFrontend>,
    documents: Vec<DocumentFrontend>,
}

#[allow(unused_variables)]
#[tuono_lib::handler]
async fn get_ia_integrated_data(req: Request, fetch: reqwest::Client) -> Response {
    let ollama_env = std::env::var("OLLAMA_API_URL").expect("Fail to read OLLAMA_API_URL env");
    let ollama_api_url: &str = &format!("{}api/tags", ollama_env); 

    let models_response = match fetch.get(ollama_api_url).send().await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to connect to Ollama API (tags) for pre-render: {}", e);
            return Response::Custom((http::StatusCode::INTERNAL_SERVER_ERROR, http::HeaderMap::new(), format!("Failed to fetch Ollama models for page: {}", e)));
        }
    };

    let models_data_raw: OllamaTagsResponse = match models_response.json().await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to parse Ollama API (tags) response for pre-render: {}", e);
            return Response::Custom((http::StatusCode::INTERNAL_SERVER_ERROR, http::HeaderMap::new(), format!("Failed to parse Ollama models response for page: {}", e)));
        }
    };

    let models_for_frontend: Vec<OllamaModelFrontend> = models_data_raw.models.into_iter().map(|model| {
        OllamaModelFrontend {
            id: model.name.clone(),
            nome: model.name,
        }
    }).collect();


    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Response::Custom((http::StatusCode::INTERNAL_SERVER_ERROR, http::HeaderMap::new(), format!("Database connection error: {}", e)));
        }
    };

    let document_rows = match client_db
        .query("SELECT id_documento, nome_arquivo FROM Documento ORDER BY nome_arquivo ASC;", &[])
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch documents from DB for IA page: {}", e);
            return Response::Custom((http::StatusCode::INTERNAL_SERVER_ERROR, http::HeaderMap::new(), format!("Failed to fetch documents: {}", e)));
        }
    };

    let documents_list: Vec<DocumentFrontend> = document_rows.into_iter().map(|row| {
        DocumentFrontend {
            id_documento: row.get("id_documento"),
            nome_arquivo: row.get("nome_arquivo"),
        }
    }).collect();

    Response::Props(Props::new(IaIntegratedPageProps {
        ollama_models: models_for_frontend,
        documents: documents_list,
    }))
}