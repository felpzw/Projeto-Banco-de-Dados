// src/routes/index.rs
use serde::{Deserialize, Serialize};
use tuono_lib::{Props, Request, Response};
use tuono_lib::axum::http;
use tuono_app::{connect_db}; // Certifique-se de que estes imports estão corretos
use chrono::NaiveDate; // Import NaiveDate for handling DATE type from PostgreSQL

#[derive(Debug, Serialize, Deserialize)]
struct Clientes {
    clientes: Vec<Cliente>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Cliente {
    id_cliente: i32,
    nome: String,
    email: String,
    telefone: String,
    data_cadastro: String, // Será convertida de chrono::NaiveDate
    endereco: String,
    cpf: Option<String>, // Agora opcional
    cnpj: Option<String>, // Agora opcional
}

#[allow(unused_variables)] // Keep if req is not used directly for query params here
#[tuono_lib::handler]
async fn get_clientes(req: Request) -> Response {
    let client_db = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Response::Custom((http::StatusCode::INTERNAL_SERVER_ERROR, http::HeaderMap::new(), format!("Database connection error: {}", e)));
        }
    };

    // Atualizada a query SQL para incluir 'email', 'endereco', e juntar com Pessoa_Fisica e Pessoa_Juridica
    // para buscar CPF ou CNPJ.
    let rows = match client_db
        .query(
            "SELECT 
                c.id_cliente, c.nome, c.email, c.telefone, c.endereco, c.data_cadastro,
                pf.cpf,
                pj.cnpj
            FROM Cliente c
            LEFT JOIN Pessoa_Fisica pf ON c.id_cliente = pf.id_cliente
            LEFT JOIN Pessoa_Juridica pj ON c.id_cliente = pj.id_cliente
            ORDER BY c.nome",
            &[],
        )
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to fetch clients from database: {}", e);
            return Response::Custom((http::StatusCode::INTERNAL_SERVER_ERROR, http::HeaderMap::new(), format!("Database query error: {}", e)));
        }
    };

    let mut clientes_list: Vec<Cliente> = Vec::new();
    for row in rows {
        let data_cadastro_pg: NaiveDate = row.get("data_cadastro");
        clientes_list.push(Cliente {
            id_cliente: row.get("id_cliente"),
            nome: row.get("nome"),
            email: row.get("email"),
            telefone: row.get("telefone"),
            data_cadastro: data_cadastro_pg.to_string(), // Convertendo Date para String
            endereco: row.get("endereco"),
            cpf: row.get("cpf"), // Pode ser None
            cnpj: row.get("cnpj"), // Pode ser None
        });
    }

    Response::Props(Props::new(Clientes {
        clientes: clientes_list,
    }))
}
