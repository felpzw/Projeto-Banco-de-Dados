use std::collections::HashMap;

use tokio_postgres::{Client, Error};
use tuono_lib::tokio;
use urlencoding;

pub async fn connect_db() -> Result<Client, Error> {
    let database_url = "host=localhost port=5432 user=usuario password=1234 dbname=banco_de_dados";
    let (client, connection) = tokio_postgres::connect(database_url, tokio_postgres::NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    Ok(client)
}

pub fn extract_query_values(query: &str) -> Result<HashMap<String, String>, String> {
    let mut params = HashMap::new();
    if query.trim().is_empty() {
        return Err("A consulta está vazia.".to_string());
    }

    let parts: Vec<&str> = query.split('&').collect();
    for part in parts {
        let kv: Vec<&str> = part.split('=').collect();
        if kv.len() != 2 || kv[0].trim().is_empty() {
            return Err(format!("Valor inválido na consulta: '{}'", part));
        }

        let key = urlencoding::decode(kv[0]).map_err(|e| format!("Failed to decode key '{}': {}", kv[0], e))?.into_owned(); // Decode key
        let value = urlencoding::decode(kv[1]).map_err(|e| format!("Failed to decode value '{}': {}", kv[1], e))?.into_owned(); // Decode value
        
        // Replace '+' with space for form-urlencoded data as per standard
        let value = value.replace('+', " ");

        params.insert(key, value);
    }

    if params.is_empty() {
        return Err("Nenhum valor encontrado na consulta.".to_string());
    }

    Ok(params)
}