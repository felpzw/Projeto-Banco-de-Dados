// src/routes/relatorios/index.rs
use serde::{Deserialize, Serialize};
use tuono_lib::{Props, Request, Response};
use tuono_lib::axum::http::{StatusCode, HeaderMap}; // Importado HeaderMap e StatusCode
use tuono_lib::axum::response::{IntoResponse, Json}; // Importado IntoResponse e Json
use tuono_app::connect_db; // Para conectar diretamente ao DB
use serde_json::json; // Importado a macro json!


// Estrutura para o relatório "Documentos por Cliente e Caso"
#[derive(Debug, Serialize, Deserialize, Clone)]
struct RelatorioDocumentosClienteCaso {
    cliente_nome: String,
    numero_processo: Option<String>,
    total_documentos: i64,
}

// Estrutura para o relatório "Total de Casos por Advogado e Status"
#[derive(Debug, Serialize, Deserialize, Clone)]
struct RelatorioCasosAdvogadoStatus {
    advogado_nome: String,
    status_descricao: String,
    total_casos: i64,
}

// Estrutura para o relatório "Total de Audiências por Cliente e Advogado"
#[derive(Debug, Serialize, Deserialize, Clone)]
struct RelatorioAudienciasClienteAdvogado {
    cliente_nome: String,
    advogado_nome: String,
    total_audiencias: i64,
}


// Propriedades da Página de Relatórios (incluindo os novos dados)
#[derive(Debug, Serialize, Deserialize, Clone)]
struct RelatoriosPageProps {
    report_data_docs_clientes_casos: Vec<RelatorioDocumentosClienteCaso>,
    report_data_casos_advogado_status: Vec<RelatorioCasosAdvogadoStatus>,
    report_data_audiencias_cliente_advogado: Vec<RelatorioAudienciasClienteAdvogado>,
}

#[allow(unused_variables)] 
#[tuono_lib::handler]
async fn get_relatorios_data(req: Request) -> Response {
    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            // Retorna um JSON de erro em caso de falha de conexão, convertido para String
            return Response::Custom((StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new(), json!({"error": format!("Database connection error: {}", e)}).to_string()));
        }
    };

    // --- 1. Relatório: Número de Documentos por Cliente e Caso ---
    // Descrição: Conta o total de documentos para cada combinação de cliente e número de processo.
    let query_docs_clientes_casos = "
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
    let rows_docs_clientes_casos = match client_db.query(query_docs_clientes_casos, &[]).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch report_data_docs_clientes_casos: {}", e);
            return Response::Custom((StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new(), json!({"error": format!("Failed to fetch report_data_docs_clientes_casos: {}", e)}).to_string()));
        }
    };
    let report_data_docs_clientes_casos: Vec<RelatorioDocumentosClienteCaso> = rows_docs_clientes_casos.into_iter().map(|row| {
        RelatorioDocumentosClienteCaso {
            cliente_nome: row.get("cliente_nome"),
            numero_processo: row.get("numero_processo"),
            total_documentos: row.get("total_documentos"),
        }
    }).collect();


    // --- 2. Relatório: Total de Casos por Advogado e Status ---
    // Descrição: Conta o número de casos para cada advogado, categorizado pelo status do caso.
    let query_casos_advogado_status = "
        SELECT
            adv.nome AS advogado_nome,
            s.descricao AS status_descricao,
            COUNT(c.id_caso) AS total_casos
        FROM Caso c
        JOIN Advogado adv ON c.id_advogado = adv.id_advogado
        JOIN Status s ON c.id_status = s.id_status
        GROUP BY adv.nome, s.descricao
        ORDER BY adv.nome, s.descricao;
    ";
    let rows_casos_advogado_status = match client_db.query(query_casos_advogado_status, &[]).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch report_data_casos_advogado_status: {}", e);
            return Response::Custom((StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new(), json!({"error": format!("Failed to fetch report_data_casos_advogado_status: {}", e)}).to_string()));
        }
    };
    let report_data_casos_advogado_status: Vec<RelatorioCasosAdvogadoStatus> = rows_casos_advogado_status.into_iter().map(|row| {
        RelatorioCasosAdvogadoStatus {
            advogado_nome: row.get("advogado_nome"),
            status_descricao: row.get("status_descricao"),
            total_casos: row.get("total_casos"),
        }
    }).collect();


    // --- 3. Relatório: Total de Audiências por Cliente e Advogado ---
    // Descrição: Conta o número de audiências realizadas para cada cliente e o advogado responsável.
    let query_audiencias_cliente_advogado = "
        SELECT
            cl.nome AS cliente_nome,
            adv.nome AS advogado_nome,
            COUNT(a.id_audiencia) AS total_audiencias
        FROM Audiencia a
        JOIN Caso c ON a.id_caso = c.id_caso
        JOIN Cliente cl ON c.id_cliente = cl.id_cliente
        JOIN Advogado adv ON c.id_advogado = adv.id_advogado
        GROUP BY cl.nome, adv.nome
        ORDER BY cl.nome, adv.nome;
    ";
    let rows_audiencias_cliente_advogado = match client_db.query(query_audiencias_cliente_advogado, &[]).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch report_data_audiencias_cliente_advogado: {}", e);
            return Response::Custom((StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new(), json!({"error": format!("Failed to fetch report_data_audiencias_cliente_advogado: {}", e)}).to_string()));
        }
    };
    let report_data_audiencias_cliente_advogado: Vec<RelatorioAudienciasClienteAdvogado> = rows_audiencias_cliente_advogado.into_iter().map(|row| {
        RelatorioAudienciasClienteAdvogado {
            cliente_nome: row.get("cliente_nome"),
            advogado_nome: row.get("advogado_nome"),
            total_audiencias: row.get("total_audiencias"),
        }
    }).collect();


    Response::Props(Props::new(RelatoriosPageProps {
        report_data_docs_clientes_casos,
        report_data_casos_advogado_status,
        report_data_audiencias_cliente_advogado,
    }))
}