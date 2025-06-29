// src/routes/relatorios/index.rs
use serde::{Deserialize, Serialize};
use tuono_lib::{Props, Request, Response};
use tuono_lib::axum::http;
use tuono_app::connect_db; // Para conectar diretamente ao DB

// Estrutura para os dados do relatório "Documentos por Cliente e Caso"
#[derive(Debug, Serialize, Deserialize, Clone)]
struct RelatorioDocumentosClienteCaso {
    cliente_nome: String,
    numero_processo: Option<String>,
    total_documentos: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RelatoriosPageProps {
    report_data_docs_clientes_casos: Vec<RelatorioDocumentosClienteCaso>,
}

#[allow(unused_variables)] 
#[tuono_lib::handler]
async fn get_relatorios_data(req: Request) -> Response {
    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Response::Custom((http::StatusCode::INTERNAL_SERVER_ERROR, http::HeaderMap::new(), format!("Database connection error: {}", e)));
        }
    };

    let query = "
        SELECT
            cl.nome AS cliente_nome,
            c.numero_processo,
            COUNT(d.id_documento) AS total_documentos
        FROM Documento d
        JOIN Caso c ON d.id_caso = c.id_caso
        JOIN Cliente cl ON c.id_cliente = cl.id_cliente
        GROUP BY cl.nome, c.numero_processo
        ORDER BY cl.nome, c.numero_processo;
    ";

    let rows = match client_db.query(query, &[]).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch report data: {}", e);
            return Response::Custom((http::StatusCode::INTERNAL_SERVER_ERROR, http::HeaderMap::new(), format!("Failed to fetch report data: {}", e)));
        }
    };

    let report_data: Vec<RelatorioDocumentosClienteCaso> = rows.into_iter().map(|row| {
        RelatorioDocumentosClienteCaso {
            cliente_nome: row.get("cliente_nome"),
            numero_processo: row.get("numero_processo"),
            total_documentos: row.get("total_documentos"),
        }
    }).collect();

    Response::Props(Props::new(RelatoriosPageProps {
        report_data_docs_clientes_casos: report_data,
    }))
}