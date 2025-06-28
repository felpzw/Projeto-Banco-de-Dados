// src/routes/documentos/index.rs
use serde::{Deserialize, Serialize};
use tuono_lib::{Props, Request, Response};
use tuono_lib::axum::http;
use tuono_app::connect_db;
use chrono::NaiveDate;

#[derive(Debug, Serialize, Deserialize)]
struct Documents {
    documents: Vec<Documento>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Documento {
    id_documento: i32,
    id_caso: i32,
    descricao: String,
    data_envio: Option<String>, // Pode ser nulo no DB
    tipo: String,
    nome_arquivo: String,
}

#[allow(unused_variables)]
#[tuono_lib::handler]
async fn get_documents(req: Request) -> Response {
    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Response::Custom((http::StatusCode::INTERNAL_SERVER_ERROR, http::HeaderMap::new(), format!("Database connection error: {}", e)));
        }
    };

    let rows = match client_db
        .query(
            "SELECT id_documento, id_caso, descricao, data_envio, tipo, nome_arquivo FROM Documento ORDER BY data_envio DESC",
            &[],
        )
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch documents from database: {}", e);
            return Response::Custom((http::StatusCode::INTERNAL_SERVER_ERROR, http::HeaderMap::new(), format!("Database query error: {}", e)));
        }
    };

    let mut documents_list: Vec<Documento> = Vec::new();
    for row in rows {
        let data_envio_pg: Option<NaiveDate> = row.get("data_envio");
        documents_list.push(Documento {
            id_documento: row.get("id_documento"),
            id_caso: row.get("id_caso"),
            descricao: row.get("descricao"),
            data_envio: data_envio_pg.map(|d| d.to_string()),
            tipo: row.get("tipo"),
            nome_arquivo: row.get("nome_arquivo"),
        });
    }

    Response::Props(Props::new(Documents {
        documents: documents_list,
    }))
}