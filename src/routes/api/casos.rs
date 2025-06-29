use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::StatusCode;
use tuono_lib::Request;
use tuono_app::{connect_db, extract_query_values};

use serde_json::{json, Value};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize}; // Necessário para as structs de payload

// --- Structs para Payload JSON (Frontend -> Backend) ---
#[derive(Debug, Serialize, Deserialize)]
struct CasoPayload {
    id_cliente: i32,
    id_advogado: i32,
    id_status: i32,
    id_vara_judicial: Option<i32>,
    id_categoria_caso: Option<i32>,
    descricao: Option<String>,
    numero_processo: Option<String>,
    data_abertura: String, // String no formato AAAA-MM-DD
    data_fechamento: Option<String>, // String no formato AAAA-MM-DD
}

#[derive(Debug, Serialize, Deserialize)]
struct CasoUpdatePayload {
    id_caso: i32, // ID do caso a ser atualizado
    id_cliente: i32,
    id_advogado: i32,
    id_status: i32,
    id_vara_judicial: Option<i32>,
    id_categoria_caso: Option<i32>,
    descricao: Option<String>,
    numero_processo: Option<String>,
    data_abertura: String, // String no formato AAAA-MM-DD
    data_fechamento: Option<String>, // String no formato AAAA-MM-DD
}


// GET /api/casos (Listar todos os casos ou um específico por ID)
#[tuono_lib::api(GET)]
async fn caso(_req: Request) -> impl IntoResponse {
    let query_string = _req.uri.query().unwrap_or("");
    let query_values_result = tuono_app::extract_query_values(query_string);

    let client_db = match tuono_app::connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)}))).into_response();
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
    ";

    // Se um ID for fornecido na query, buscar um caso específico
    let mut is_specific_id_requested = false;
    let rows_result = if let Ok(values) = query_values_result {
        if let Some(id_str) = values.get("id").cloned() {
            is_specific_id_requested = true;
            let id = match id_str.parse::<i32>() {
                Ok(id) => id,
                Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID do caso deve ser um número inteiro."}))).into_response(),
            };
            match client_db
                .query(&format!("{} WHERE c.id_caso = $1;", base_query), &[&id])
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to fetch specific case: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to fetch specific case: {}", e)}))).into_response();
                }
            }
        } else {
            // Nenhum ID na query, retornar todos
            match client_db
                .query(&format!("{} ORDER BY c.data_abertura DESC;", base_query), &[])
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to fetch all cases: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to fetch all cases: {}", e)}))).into_response();
                }
            }
        }
    } else {
        // Erro ao extrair query values, então retornar todos os casos
        match client_db
            .query(&format!("{} ORDER BY c.data_abertura DESC;", base_query), &[])
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to fetch all cases: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to fetch all cases: {}", e)}))).into_response();
            }
        }
    };

    let cases: Vec<Value> = rows_result.into_iter().map(|row| {
        let data_abertura: NaiveDate = row.get("data_abertura");
        let data_fechamento: Option<NaiveDate> = row.get("data_fechamento");

        json!({
            "id_caso": row.get::<_, i32>("id_caso"),
            "descricao": row.get::<_, Option<String>>("descricao"),
            "numero_processo": row.get::<_, Option<String>>("numero_processo"),
            "data_abertura": data_abertura.to_string(),
            "data_fechamento": data_fechamento.map(|d| d.to_string()),
            // Informações do Cliente
            "id_cliente": row.get::<_, i32>("id_cliente"),
            "cliente_nome": row.get::<_, String>("cliente_nome"),
            "cliente_email": row.get::<_, Option<String>>("cliente_email"),
            // Informações do Advogado
            "id_advogado": row.get::<_, i32>("id_advogado"),
            "advogado_nome": row.get::<_, String>("advogado_nome"),
            "advogado_oab": row.get::<_, String>("advogado_oab"),
            // Informações do Status
            "id_status": row.get::<_, i32>("id_status"),
            "status_descricao": row.get::<_, String>("status_descricao"),
            // Informações da Vara Judicial (podem ser nulas)
            "id_vara_judicial": row.get::<_, Option<i32>>("id_vara_judicial"),
            "nome_vara": row.get::<_, Option<String>>("nome_vara"),
            // Informações da Categoria do Caso (podem ser nulas)
            "id_categoria_caso": row.get::<_, Option<i32>>("id_categoria_caso"),
            "categoria_descricao": row.get::<_, Option<String>>("categoria_descricao"),
        })
    }).collect();

    if is_specific_id_requested {
        if cases.is_empty() {
            return (StatusCode::NOT_FOUND, Json(json!({"error": "Caso não encontrado."}))).into_response();
        } else {
            return (StatusCode::OK, Json(cases[0].clone())).into_response();
        }
    }
    Json(json!(cases)).into_response()
}



#[tuono_lib::api(POST)]
async fn create_caso(_req: Request) -> impl IntoResponse {
    let payload: CasoPayload = match _req.body() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to read/parse JSON body from Request: {:?}", e);
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid request body or JSON parsing error: {:?}", e)})));
        }
    };

    let id_cliente = payload.id_cliente;
    let id_advogado = payload.id_advogado;
    let id_status = payload.id_status;
    let id_vara_judicial = payload.id_vara_judicial;
    let id_categoria_caso = payload.id_categoria_caso;
    let descricao = payload.descricao;
    let numero_processo = payload.numero_processo;
    let data_abertura = match NaiveDate::parse_from_str(&payload.data_abertura, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Data de abertura inválida. Use o formato AAAA-MM-DD."}))),
    };
    let data_fechamento = payload.data_fechamento.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());


    let client_db = match tuono_app::connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)})));
        }
    };

    // Validação de FKs
    let cliente_exists = client_db.query_opt("SELECT 1 FROM Cliente WHERE id_cliente = $1;", &[&id_cliente]).await.map_err(|e| format!("DB error checking Cliente: {}", e)).unwrap_or_default().is_some();
    let advogado_exists = client_db.query_opt("SELECT 1 FROM Advogado WHERE id_advogado = $1;", &[&id_advogado]).await.map_err(|e| format!("DB error checking Advogado: {}", e)).unwrap_or_default().is_some();
    let status_exists = client_db.query_opt("SELECT 1 FROM Status WHERE id_status = $1;", &[&id_status]).await.map_err(|e| format!("DB error checking Status: {}", e)).unwrap_or_default().is_some();
    
    if !cliente_exists { return (StatusCode::BAD_REQUEST, Json(json!({"error": "Cliente com o ID fornecido não existe."}))); }
    if !advogado_exists { return (StatusCode::BAD_REQUEST, Json(json!({"error": "Advogado com o ID fornecido não existe."}))); }
    if !status_exists { return (StatusCode::BAD_REQUEST, Json(json!({"error": "Status com o ID fornecido não existe."}))); }

    if let Some(id_vj) = id_vara_judicial {
        let vj_exists = client_db.query_opt("SELECT 1 FROM Vara_Judicial WHERE id_vara_judicial = $1;", &[&id_vj]).await.map_err(|e| format!("DB error checking Vara_Judicial: {}", e)).unwrap_or_default().is_some();
        if !vj_exists { return (StatusCode::BAD_REQUEST, Json(json!({"error": "Vara Judicial com o ID fornecido não existe."}))); }
    }
    if let Some(id_cc) = id_categoria_caso {
        let cc_exists = client_db.query_opt("SELECT 1 FROM Categoria_caso WHERE id_categoria_caso = $1;", &[&id_cc]).await.map_err(|e| format!("DB error checking Categoria_caso: {}", e)).unwrap_or_default().is_some();
        if !cc_exists { return (StatusCode::BAD_REQUEST, Json(json!({"error": "Categoria de Caso com o ID fornecido não existe."}))); }
    }


    let insert_query = "
        INSERT INTO Caso (id_cliente, id_advogado, id_status, id_vara_judicial, id_categoria_caso, descricao, numero_processo, data_abertura, data_fechamento)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id_caso;
    ";

    let rows = match client_db.query(
        insert_query,
        &[&id_cliente, &id_advogado, &id_status, &id_vara_judicial, &id_categoria_caso, &descricao, &numero_processo, &data_abertura, &data_fechamento],
    ).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to insert Caso: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to create case: {}", e)})));
        }
    };

    let id_caso_new: i32 = rows[0].get("id_caso");

    (StatusCode::CREATED, Json(json!({"message": "Caso jurídico criado com sucesso", "id_caso": id_caso_new})))
}


#[tuono_lib::api(PUT)]
async fn update_caso(_req: Request) -> impl IntoResponse {
    let payload: CasoUpdatePayload = match _req.body() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to read/parse JSON body from Request: {:?}", e);
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid request body or JSON parsing error: {:?}", e)})));
        }
    };

    let id_caso = payload.id_caso;
    let id_cliente = payload.id_cliente;
    let id_advogado = payload.id_advogado;
    let id_status = payload.id_status;
    let id_vara_judicial = payload.id_vara_judicial;
    let id_categoria_caso = payload.id_categoria_caso;
    let descricao = payload.descricao;
    let numero_processo = payload.numero_processo;
    let data_abertura = match NaiveDate::parse_from_str(&payload.data_abertura, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Data de abertura inválida. Use o formato AAAA-MM-DD."}))),
    };
    let data_fechamento = payload.data_fechamento.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());


    let client_db = match tuono_app::connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)})));
        }
    };

    // Validação de FKs
    let cliente_exists = client_db.query_opt("SELECT 1 FROM Cliente WHERE id_cliente = $1;", &[&id_cliente]).await.map_err(|e| format!("DB error checking Cliente: {}", e)).unwrap_or_default().is_some();
    let advogado_exists = client_db.query_opt("SELECT 1 FROM Advogado WHERE id_advogado = $1;", &[&id_advogado]).await.map_err(|e| format!("DB error checking Advogado: {}", e)).unwrap_or_default().is_some();
    let status_exists = client_db.query_opt("SELECT 1 FROM Status WHERE id_status = $1;", &[&id_status]).await.map_err(|e| format!("DB error checking Status: {}", e)).unwrap_or_default().is_some();
    
    if !cliente_exists { return (StatusCode::BAD_REQUEST, Json(json!({"error": "Cliente com o ID fornecido não existe."}))); }
    if !advogado_exists { return (StatusCode::BAD_REQUEST, Json(json!({"error": "Advogado com o ID fornecido não existe."}))); }
    if !status_exists { return (StatusCode::BAD_REQUEST, Json(json!({"error": "Status com o ID fornecido não existe."}))); }

    if let Some(id_vj) = id_vara_judicial {
        let vj_exists = client_db.query_opt("SELECT 1 FROM Vara_Judicial WHERE id_vara_judicial = $1;", &[&id_vj]).await.map_err(|e| format!("DB error checking Vara_Judicial: {}", e)).unwrap_or_default().is_some();
        if !vj_exists { return (StatusCode::BAD_REQUEST, Json(json!({"error": "Vara Judicial com o ID fornecido não existe."}))); }
    }
    if let Some(id_cc) = id_categoria_caso {
        let cc_exists = client_db.query_opt("SELECT 1 FROM Categoria_caso WHERE id_categoria_caso = $1;", &[&id_cc]).await.map_err(|e| format!("DB error checking Categoria_caso: {}", e)).unwrap_or_default().is_some();
        if !cc_exists { return (StatusCode::BAD_REQUEST, Json(json!({"error": "Categoria de Caso com o ID fornecido não existe."}))); }
    }

    let update_query = "
        UPDATE Caso SET
            id_cliente = $1,
            id_advogado = $2,
            id_status = $3,
            id_vara_judicial = $4,
            id_categoria_caso = $5,
            descricao = $6,
            numero_processo = $7,
            data_abertura = $8,
            data_fechamento = $9
        WHERE id_caso = $10;
    ";

    let rows_affected = match client_db.execute(
        update_query,
        &[&id_cliente, &id_advogado, &id_status, &id_vara_judicial, &id_categoria_caso, &descricao, &numero_processo, &data_abertura, &data_fechamento, &id_caso],
    ).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to update Caso: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update case: {}", e)})));
        }
    };

    if rows_affected > 0 {
        (StatusCode::OK, Json(json!({"message": "Caso jurídico atualizado com sucesso."})))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Caso jurídico não encontrado."})))
    }
}


// DELETE /api/casos (Excluir caso)
#[tuono_lib::api(DELETE)]
async fn delete_caso(_req: Request) -> impl IntoResponse {
    let query_string = _req.uri.query().unwrap_or("");
    let query_values = match tuono_app::extract_query_values(query_string) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to extract query values for DELETE (Caso): {}", e);
            return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("Invalid query parameters: {}", e)})));
        }
    };

    let id_caso_str = match query_values.get("id") {
        Some(id) => id,
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID do caso é obrigatório."}))),
    };
    let id_caso = match id_caso_str.parse::<i32>() {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "ID do caso deve ser um número inteiro."}))),
    };

    let client_db = match tuono_app::connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)})));
        }
    };

    let delete_query = "DELETE FROM Caso WHERE id_caso = $1;";
    let rows_affected = match client_db.execute(delete_query, &[&id_caso]).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to delete Caso: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to delete case: {}", e)})));
        }
    };

    if rows_affected > 0 {
        (StatusCode::OK, Json(json!({"message": "Caso jurídico excluído com sucesso."})))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Caso jurídico não encontrado."})))
    }
}