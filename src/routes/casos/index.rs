// src/routes/casos/index.rs
use serde::{Deserialize, Serialize};
use tuono_lib::{Props, Request, Response};
use tuono_lib::axum::http;
use tuono_app::connect_db;
use chrono::NaiveDate;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Case {
    id_caso: i32,
    descricao: Option<String>,
    numero_processo: Option<String>,
    data_abertura: String,
    data_fechamento: Option<String>,

    id_cliente: i32,
    cliente_nome: String,
    cliente_email: Option<String>,

    id_advogado: i32,
    advogado_nome: String,
    advogado_oab: String,

    id_status: i32,
    status_descricao: String,

    id_vara_judicial: Option<i32>,
    nome_vara: Option<String>,

    id_categoria_caso: Option<i32>,
    categoria_descricao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CasesPageProps {
    cases: Vec<Case>,
}

#[allow(unused_variables)]
#[tuono_lib::handler]
async fn get_cases(req: Request) -> Response {
    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Response::Custom((http::StatusCode::INTERNAL_SERVER_ERROR, http::HeaderMap::new(), format!("Database connection error: {}", e)));
        }
    };

    let base_query = "
        SELECT
            c.id_caso, c.descricao, c.numero_processo, c.data_abertura, c.data_fechamento,
            cl.id_cliente, cl.nome AS cliente_nome, cl.email AS cliente_email,
            adv.id_advogado, adv.nome AS advogado_nome, adv.oab AS advogado_oab,
            s.id_status, s.descricao AS status_descricao,
            vj.id_vara_judicial, vj.nome_vara,
            cc.id_categoria_caso, cc.descricao AS categoria_descricao
        FROM Caso c
        INNER JOIN Cliente cl ON c.id_cliente = cl.id_cliente
        INNER JOIN Advogado adv ON c.id_advogado = adv.id_advogado
        INNER JOIN Status s ON c.id_status = s.id_status
        LEFT JOIN Vara_Judicial vj ON c.id_vara_judicial = vj.id_vara_judicial
        LEFT JOIN Categoria_caso cc ON c.id_categoria_caso = cc.id_categoria_caso
        ORDER BY c.data_abertura DESC;
    ";

    let rows = match client_db
        .query(base_query, &[])
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch cases from database: {}", e);
            return Response::Custom((http::StatusCode::INTERNAL_SERVER_ERROR, http::HeaderMap::new(), format!("Database query error: {}", e)));
        }
    };

    let mut cases_list: Vec<Case> = Vec::new();
    for row in rows {
        let data_abertura_pg: NaiveDate = row.get("data_abertura");
        let data_fechamento_pg: Option<NaiveDate> = row.get("data_fechamento");

        cases_list.push(Case {
            id_caso: row.get("id_caso"),
            descricao: row.get("descricao"),
            numero_processo: row.get("numero_processo"),
            data_abertura: data_abertura_pg.to_string(),
            data_fechamento: data_fechamento_pg.map(|d| d.to_string()),
            
            id_cliente: row.get("id_cliente"),
            cliente_nome: row.get("cliente_nome"),
            cliente_email: row.get("cliente_email"),
            
            id_advogado: row.get("id_advogado"),
            advogado_nome: row.get("advogado_nome"),
            advogado_oab: row.get("advogado_oab"),
            
            id_status: row.get("id_status"),
            status_descricao: row.get("status_descricao"),
            
            id_vara_judicial: row.get("id_vara_judicial"),
            nome_vara: row.get("nome_vara"),
            
            id_categoria_caso: row.get("id_categoria_caso"),
            categoria_descricao: row.get("categoria_descricao"),
        });
    }

    Response::Props(Props::new(CasesPageProps {
        cases: cases_list,
    }))
}