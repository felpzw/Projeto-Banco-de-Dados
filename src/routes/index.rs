// src/routes/index.rs
use serde::{Deserialize, Serialize};
use reqwest::Client;
use tuono_lib::{Props, Request, Response};
use tuono_lib::tokio;
use tokio_postgres::NoTls;

const HEALTH_CHECK: &str = "http://localhost:3000/api/health_check";

const DATABASE_URL: &str = "host=localhost port=5432 user=usuario password=1234 dbname=banco_de_dados";



#[derive(Debug, Serialize, Deserialize)]
struct Status {
    api_status: String,
    db_status: String,
}

#[allow(unused_imports)]
#[tuono_lib::handler]
async fn get_all_pokemons(_req: Request, fetch: Client) -> Response {
    let api_check = {
        let response = fetch.get(HEALTH_CHECK).send().await;
        match response {
            Ok(res) => res.status().to_string(),
            Err(_) => "Error".to_string(),
        }

    };

    let db_check = {
        match tokio_postgres::connect(DATABASE_URL, NoTls).await {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("Erro na conexão do banco de dados (background task): {}", e);
                    }
                });
                match client.query_one("SELECT 1", &[]).await {
                    Ok(_) => "OK".to_string(),
                    Err(e) => format!("Database query error: {}", e),
                }
            },
            Err(e) => format!("Database connection failed: {}", e), // Lida com falha na conexão
        }
    };
    

    //let db_check = "OK".to_string();



    Response::Props(Props::new(Status {
        api_status: api_check,
        db_status: db_check,
    }))
}