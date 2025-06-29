use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::StatusCode;
use tuono_lib::Request;
use tuono_app::{connect_db, extract_query_values}; // Re-importando para qualified calls

use serde_json::{json, Value};
use chrono::NaiveDate;  

#[tuono_lib::api(GET)]  
async fn cliente(_req: Request) -> Json<Value> { 
    let query_string = _req.uri.query().unwrap_or(""); 
    let query_values_result = tuono_app::extract_query_values(query_string); // Obtenha o resultado da análise da query

    let client_db = match tuono_app::connect_db().await { 
        Ok(client) => client,
        Err(e) => { 
            eprintln!("Failed to connect to database: {}", e); 
            return Json(json!({ 
                "error": format!("Database connection error: {}", e) 
            }));
        }
    };

    // Se houver um parâmetro 'id' na query string, buscar um cliente específico
    if let Ok(query_values) = &query_values_result { // `query_values` é uma referência temporária a HashMap<String, String>
        if let Some(id_str) = query_values.get("id") { // Verifica a presença de um ID específico
            let id = match id_str.parse::<i32>() { 
                Ok(id) => id,
                Err(_) => { 
                    return Json(json!({ 
                        "error": "ID parameter must be an integer." 
                    }));
                }
            };

            let rows = match client_db
                .query(
                    "SELECT 
                        c.id_cliente, c.nome, c.email, c.telefone, c.endereco, c.data_cadastro,
                        pf.cpf,
                        pj.cnpj
                    FROM Cliente c
                    LEFT JOIN Pessoa_Fisica pf ON c.id_cliente = pf.id_cliente
                    LEFT JOIN Pessoa_Juridica pj ON c.id_cliente = pj.id_cliente
                    WHERE c.id_cliente = $1;",
                    &[&id], 
                )
                .await
            {
                Ok(rows) => rows,
                Err(e) => { 
                    eprintln!("Failed to execute query: {}", e); 
                    return Json(json!({ 
                        "error": format!("Failed to fetch client: {}", e) 
                    }));
                }
            };

            if rows.is_empty() { 
                return Json(json!({ 
                    "error": "Client not found." 
                }));
            }

            let row = &rows[0]; 
            return Json( // Retorna imediatamente para o cliente específico
                json!({ 
                    "id_cliente": row.get::<_, i32>("id_cliente"), 
                    "nome": row.get::<_, String>("nome"), 
                    "email": row.get::<_, Option<String>>("email").unwrap_or_else(|| "Nao identificado".to_string()), 
                    "telefone": row.get::<_, Option<String>>("telefone").unwrap_or_else(|| "Nao identificado".to_string()), 
                    "endereco": row.get::<_, Option<String>>("endereco").unwrap_or_else(|| "Nao identificado".to_string()), 
                    "data_cadastro": row 
                        .get::<_, Option<NaiveDate>>("data_cadastro") 
                        .map(|d| d.to_string()) 
                        .unwrap_or_else(|| "Nao identificado".to_string()), 
                    "cpf": row.get::<_, Option<String>>("cpf"),
                    "cnpj": row.get::<_, Option<String>>("cnpj"),
                })
            );
        }
    }

    // Se NÃO houver um parâmetro 'id' (ou se houver um erro na extração mas a query_string não está vazia),
    // ou se a query_string estiver vazia, retorna a lista completa de clientes para dropdowns
    let rows = match client_db
        .query(
            "SELECT id_cliente, nome FROM Cliente ORDER BY nome ASC;", 
            &[], 
        )
        .await
    {
        Ok(rows) => rows,
        Err(e) => { 
            eprintln!("Failed to fetch all clients for lookup: {}", e); 
            return Json(json!({ 
                "error": format!("Failed to fetch clients list: {}", e) 
            }));
        }
    };

    let clients_list: Vec<Value> = rows.into_iter().map(|row| {
        json!({
            "id_cliente": row.get::<_, i32>("id_cliente"),
            "nome": row.get::<_, String>("nome"),
        })
    }).collect();

    Json(json!(clients_list))
}

#[tuono_lib::api(POST)]
async fn create_client(_req: Request) -> impl IntoResponse { 
    let query_string = _req.uri.query().unwrap_or(""); 
    println!("Query Recebida para POST: {}", query_string);
    let query_values = match extract_query_values(query_string) { 
        Ok(value) => value,
        Err(e) => { 
            eprintln!("Failed to extract query values for POST: {}", e); 
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid query parameters: {}", e)}))); 
        }
    };

    let nome = match query_values.get("nome") { 
        Some(n) => n.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Nome is required."}))), 
    };
    let email = match query_values.get("email") { 
        Some(e) => e.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Email is required."}))), 
    };
    let telefone = query_values.get("telefone").cloned().unwrap_or_else(|| "Nao identificado".to_string()); 
    let endereco = query_values.get("endereco").cloned().unwrap_or_else(|| "Nao identificado".to_string()); 
    let tipo_cliente = match query_values.get("tipoCliente") { 
        Some(tc) => tc.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "tipoCliente is required (fisica or juridica)."}))), 
    };
    let cpf = query_values.get("cpf").cloned(); 
    let cnpj = query_values.get("cnpj").cloned(); 

    let mut client_db = match connect_db().await { 
        Ok(client) => client,
        Err(e) => { 
            eprintln!("Failed to connect to database: {}", e); 
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)}))); 
        }
    };

    let transaction = match client_db.transaction().await { 
        Ok(tx) => tx,
        Err(e) => { 
            eprintln!("Failed to start transaction: {}", e); 
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to start transaction: {}", e)}))); 
        }
    };

    let insert_client_query = "INSERT INTO Cliente (nome, email, telefone, endereco, data_cadastro) VALUES ($1, $2, $3, $4, CURRENT_DATE) RETURNING id_cliente;"; 
    let client_rows = match transaction.query(insert_client_query, &[&nome, &email, &telefone, &endereco]).await { 
        Ok(rows) => rows,
        Err(e) => { 
            eprintln!("Failed to insert into Cliente: {}", e); 
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert client: {}", e)}))); 
        }
    };

    let client_id: i32 = client_rows[0].get("id_cliente"); 

    if tipo_cliente == "fisica" { 
        if let Some(c) = cpf { 
            let insert_pf_query = "INSERT INTO Pessoa_Fisica (id_cliente, cpf) VALUES ($1, $2);"; 
            if let Err(e) = transaction.execute(insert_pf_query, &[&client_id, &c]).await { 
                eprintln!("Failed to insert into Pessoa_Fisica: {}", e); 
                let _ = transaction.rollback().await; 
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert physical person details: {}", e)}))); 
            }
        } else { 
            let _ = transaction.rollback().await; 
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "CPF is required for Pessoa Física."}))); 
        }
    } else if tipo_cliente == "juridica" { 
        if let Some(c) = cnpj { 
            let insert_pj_query = "INSERT INTO Pessoa_Juridica (id_cliente, cnpj) VALUES ($1, $2);"; 
            if let Err(e) = transaction.execute(insert_pj_query, &[&client_id, &c]).await { 
                eprintln!("Failed to insert into Pessoa_Juridica: {}", e); 
                let _ = transaction.rollback().await;
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert legal person details: {}", e)}))); 
            }
        } else { 
            let _ = transaction.rollback().await; 
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "CNPJ is required for Pessoa Jurídica."}))); 
        }
    } else { 
        let _ = transaction.rollback().await; 
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid tipoCliente provided."}))); 
    }

    if let Err(e) = transaction.commit().await { 
        eprintln!("Failed to commit transaction: {}", e); 
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to commit transaction: {}", e)}))); 
    }

    (StatusCode::CREATED, Json(json!({"message": "Client created successfully", "id_cliente": client_id}))) 
}

#[tuono_lib::api(PUT)] // New PUT endpoint for updating clients
async fn update_client(_req: Request) -> impl IntoResponse {
    let query_string = _req.uri.query().unwrap_or(""); 
    println!("Query Recebida para PUT: {}", query_string);
    let query_values = match extract_query_values(query_string) { 
        Ok(value) => value,
        Err(e) => { 
            eprintln!("Failed to extract query values for PUT: {}", e); 
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid query parameters: {}", e)}))); 
        }
    };

    let id_str = match query_values.get("id") { 
        Some(id) => id,
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID parameter is required."}))), 
    };

    let id = match id_str.parse::<i32>() { 
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID parameter must be an integer."}))), 
    };

    let nome = match query_values.get("nome") { 
        Some(n) => n.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Nome is required."}))), 
    };
    let email = match query_values.get("email") { 
        Some(e) => e.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Email is required."}))), 
    };
    let telefone = query_values.get("telefone").cloned().unwrap_or_else(|| "Nao identificado".to_string()); 
    let endereco = query_values.get("endereco").cloned().unwrap_or_else(|| "Nao identificado".to_string()); 
    let tipo_cliente = match query_values.get("tipoCliente") { 
        Some(tc) => tc.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "tipoCliente is required (fisica or juridica)."}))), 
    };
    let original_tipo_cliente = query_values.get("originalTipoCliente").cloned().unwrap_or_else(|| "".to_string()); // Capture original type

    let cpf = query_values.get("cpf").cloned(); 
    let cnpj = query_values.get("cnpj").cloned(); 

    let mut client_db = match connect_db().await { 
        Ok(client) => client,
        Err(e) => { 
            eprintln!("Failed to connect to database: {}", e); 
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)}))); 
        }
    };

    let transaction = match client_db.transaction().await { 
        Ok(tx) => tx,
        Err(e) => { 
            eprintln!("Failed to start transaction: {}", e); 
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to start transaction: {}", e)}))); 
        }
    };

    // 1. Update Cliente table
    let update_client_query = "UPDATE Cliente SET nome = $1, email = $2, telefone = $3, endereco = $4 WHERE id_cliente = $5;";
    if let Err(e) = transaction.execute(update_client_query, &[&nome, &email, &telefone, &endereco, &id]).await {
        eprintln!("Failed to update Cliente: {}", e);
        let _ = transaction.rollback().await;
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update client details: {}", e)})));
    }

    // 2. Handle type-specific table updates/deletions/insertions
    if original_tipo_cliente == "fisica" && tipo_cliente == "juridica" {
        // Change from Fisica to Juridica: Delete from Fisica, Insert into Juridica
        if let Err(e) = transaction.execute("DELETE FROM Pessoa_Fisica WHERE id_cliente = $1;", &[&id]).await {
            eprintln!("Failed to delete from Pessoa_Fisica during type change: {}", e);
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to change client type (delete fisica): {}", e)})));
        }
        if let Some(c) = cnpj {
            if let Err(e) = transaction.execute("INSERT INTO Pessoa_Juridica (id_cliente, cnpj) VALUES ($1, $2);", &[&id, &c]).await {
                eprintln!("Failed to insert into Pessoa_Juridica during type change: {}", e);
                let _ = transaction.rollback().await;
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to change client type (insert juridica): {}", e)})));
            }
        } else {
            let _ = transaction.rollback().await;
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "CNPJ is required when changing to Pessoa Jurídica."})));
        }
    } else if original_tipo_cliente == "juridica" && tipo_cliente == "fisica" {
        // Change from Juridica to Fisica: Delete from Juridica, Insert into Fisica
        if let Err(e) = transaction.execute("DELETE FROM Pessoa_Juridica WHERE id_cliente = $1;", &[&id]).await {
            eprintln!("Failed to delete from Pessoa_Juridica during type change: {}", e);
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to change client type (delete juridica): {}", e)})));
        }
        if let Some(c) = cpf {
            if let Err(e) = transaction.execute("INSERT INTO Pessoa_Fisica (id_cliente, cpf) VALUES ($1, $2);", &[&id, &c]).await {
                eprintln!("Failed to insert into Pessoa_Fisica during type change: {}", e);
                let _ = transaction.rollback().await;
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to change client type (insert fisica): {}", e)})));
            }
        } else {
            let _ = transaction.rollback().await;
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "CPF is required when changing to Pessoa Física."})));
        }
    } else if original_tipo_cliente == tipo_cliente {
        // Type remains the same: Just update the specific type table
        if tipo_cliente == "fisica" {
            if let Some(c) = cpf {
                if let Err(e) = transaction.execute("UPDATE Pessoa_Fisica SET cpf = $1 WHERE id_cliente = $2;", &[&c, &id]).await {
                    eprintln!("Failed to update Pessoa_Fisica: {}", e);
                    let _ = transaction.rollback().await;
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update physical person details: {}", e)})));
                }
            } else {
                let _ = transaction.rollback().await;
                return (StatusCode::BAD_REQUEST, Json(json!({"error": "CPF is required for Pessoa Física."})));
            }
        } else if tipo_cliente == "juridica" {
            if let Some(c) = cnpj {
                if let Err(e) = transaction.execute("UPDATE Pessoa_Juridica SET cnpj = $1 WHERE id_cliente = $2;", &[&c, &id]).await {
                    eprintln!("Failed to update Pessoa_Juridica: {}", e);
                    let _ = transaction.rollback().await;
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update legal person details: {}", e)})));
                }
            } else {
                let _ = transaction.rollback().await;
                return (StatusCode::BAD_REQUEST, Json(json!({"error": "CNPJ is required for Pessoa Jurídica."})));
            }
        }
    } else {
        // Handle cases where original_tipo_cliente might be empty (e.g., client only in Cliente table initially)
        // Or if new type is invalid (already caught by earlier validation)
        // For now, if no type change or specific type provided, we assume no specific type update needed.
        // If a client was created without CPF/CNPJ, and then one is added, this would need to insert.
        // This scenario is not explicitly handled here, assuming clients are always one type or the other.
        if tipo_cliente == "fisica" && original_tipo_cliente.is_empty() {
             if let Some(c) = cpf {
                if let Err(e) = transaction.execute("INSERT INTO Pessoa_Fisica (id_cliente, cpf) VALUES ($1, $2);", &[&id, &c]).await {
                    eprintln!("Failed to insert Pessoa_Fisica for initially untyped client: {}", e);
                    let _ = transaction.rollback().await;
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to add physical person details: {}", e)})));
                }
            } else {
                let _ = transaction.rollback().await;
                return (StatusCode::BAD_REQUEST, Json(json!({"error": "CPF is required for Pessoa Física."})));
            }
        } else if tipo_cliente == "juridica" && original_tipo_cliente.is_empty() {
            if let Some(c) = cnpj {
                if let Err(e) = transaction.execute("INSERT INTO Pessoa_Juridica (id_cliente, cnpj) VALUES ($1, $2);", &[&id, &c]).await {
                    eprintln!("Failed to insert Pessoa_Juridica for initially untyped client: {}", e);
                    let _ = transaction.rollback().await;
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to add legal person details: {}", e)})));
                }
            } else {
                let _ = transaction.rollback().await;
                return (StatusCode::BAD_REQUEST, Json(json!({"error": "CNPJ is required for Pessoa Jurídica."})));
            }
        }
    }


    if let Err(e) = transaction.commit().await { 
        eprintln!("Failed to commit transaction: {}", e); 
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to commit transaction: {}", e)}))); 
    }

    (StatusCode::OK, Json(json!({"message": "Client updated successfully"}))) 
}

#[tuono_lib::api(DELETE)]
async fn delete_client(_req: Request) -> impl IntoResponse {
    let query_string = _req.uri.query().unwrap_or(""); 
    println!("Query Recebida para DELETE: {}", query_string);
    let query_values = match extract_query_values(query_string) { 
        Ok(values) => values,
        Err(e) => { 
            eprintln!("Failed to extract query values for DELETE: {}", e); 
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid query parameters: {}", e)}))); 
        }
    };

    let id_str = match query_values.get("id") { 
        Some(id) => id,
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID parameter is required."}))), 
    };

    let id = match id_str.parse::<i32>() { 
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID parameter must be an integer."}))), 
    };

    let mut client_db = match connect_db().await { 
        Ok(client) => client,
        Err(e) => { 
            eprintln!("Failed to connect to database: {}", e); 
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)}))); 
        }
    };

    let transaction = match client_db.transaction().await { 
        Ok(tx) => tx,
        Err(e) => { 
            eprintln!("Failed to start transaction: {}", e); 
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to start transaction: {}", e)}))); 
        }
    };

    // Primeiro, determine o tipo de cliente (Pessoa_Fisica ou Pessoa_Juridica)
    let client_type_query = "SELECT cpf, cnpj FROM Cliente c LEFT JOIN Pessoa_Fisica pf ON c.id_cliente = pf.id_cliente LEFT JOIN Pessoa_Juridica pj ON c.id_cliente = pj.id_cliente WHERE c.id_cliente = $1;";
    let client_rows = match transaction.query(client_type_query, &[&id]).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to query client type: {}", e);
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to determine client type: {}", e)})));
        }
    };

    if client_rows.is_empty() {
        let _ = transaction.rollback().await;
        return (StatusCode::NOT_FOUND, Json(json!({"error": "Client not found."})));
    }

    let row = &client_rows[0];
    let cpf: Option<String> = row.get("cpf");
    let cnpj: Option<String> = row.get("cnpj");

    if cpf.is_some() {
        // É uma Pessoa Física
        match transaction.execute("DELETE FROM pessoa_fisica WHERE id_cliente = $1;", &[&id]).await {
            Ok(_) => println!("Pessoa Física deleted successfully."),
            Err(e) => {
                eprintln!("Failed to delete from Pessoa_Fisica: {}", e);
                let _ = transaction.rollback().await;
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to delete physical person details: {}", e)})));
            }
        };
    } else if cnpj.is_some() {
        // É uma Pessoa Jurídica
        match transaction.execute("DELETE FROM pessoa_juridica WHERE id_cliente = $1;", &[&id]).await {
            Ok(_) => println!("Pessoa Jurídica deleted successfully."),
            Err(e) => {
                eprintln!("Failed to delete from Pessoa_Juridica: {}", e);
                let _ = transaction.rollback().await;
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to delete legal person details: {}", e)})));
            }
        };
    } else {
        // Cliente não é nem Pessoa Física nem Jurídica (pode ser um erro de dados ou um cliente que só existe na tabela Cliente)
        println!("Client is neither Pessoa Física nor Pessoa Jurídica. Proceeding to delete from Cliente table.");
    }

    // Finalmente, delete da tabela Cliente
    let delete_client_query = "DELETE FROM Cliente WHERE id_cliente = $1;";
    match transaction.execute(delete_client_query, &[&id]).await { 
        Ok(rows_affected) => { 
            if rows_affected > 0 { 
                match transaction.commit().await {
                    Ok(_) => (StatusCode::OK, Json(json!({"message": "Client deleted successfully."}))),
                    Err(e) => {
                        eprintln!("Failed to commit transaction: {}", e);
                        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to commit transaction: {}", e)})))
                    }
                }
            } else { 
                let _ = transaction.rollback().await;
                (StatusCode::NOT_FOUND, Json(json!({"error": "Client not found or already deleted from main table."}))) 
            }
        },
        Err(e) => { 
            eprintln!("Failed to delete client from main table: {}", e); 
            let _ = transaction.rollback().await;
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to delete client from main table: {}", e)}))) 
        }
    }
}