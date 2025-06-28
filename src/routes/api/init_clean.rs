use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::{StatusCode};
use tuono_lib::Request;
use serde_json::{json, Value};

use tuono_app::{connect_db};

use chrono::{NaiveDate, NaiveTime};


#[tuono_lib::api(DELETE)]
async fn clean(_req: Request) -> impl IntoResponse {
    let client = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            );
        }
    };

    // Consulta todas as tabelas no schema `public`
    let query_tables = "
        SELECT tablename FROM pg_tables
        WHERE schemaname = 'public';
    ";

    let rows = match client.query(query_tables, &[]).await {
        Ok(rows) => rows,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Query error: {}", e)})),
            );
        }
    };

    for row in rows {
        let tablename: &str = row.get("tablename");
        let drop_query = format!("DROP TABLE IF EXISTS {} CASCADE;", tablename);

        if let Err(e) = client.execute(&drop_query, &[]).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to drop table '{}': {}", tablename, e)})),
            );
        }
    }

    (StatusCode::OK, Json(json!({"message": "All tables dropped successfully."})))
}


#[tuono_lib::api(POST)]
async fn init(_req: Request) -> impl IntoResponse {
    let client = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})));
        }
    };

    if let Err(e) = client.batch_execute(
        "
        CREATE TABLE Cliente (
            id_cliente SERIAL PRIMARY KEY,
            nome VARCHAR(255) NOT NULL,
            email VARCHAR(255),
            telefone VARCHAR(20),
            endereco TEXT,
            data_cadastro DATE
        );

        CREATE TABLE Pessoa_Fisica (
            id_cliente INTEGER PRIMARY KEY REFERENCES Cliente(id_cliente),
            cpf VARCHAR(14) UNIQUE NOT NULL
        );

        CREATE TABLE Pessoa_Juridica (
            id_cliente INTEGER PRIMARY KEY REFERENCES Cliente(id_cliente),
            cnpj VARCHAR(18) UNIQUE NOT NULL
        );

        CREATE TABLE Advogado (
            id_advogado SERIAL PRIMARY KEY,
            nome VARCHAR(255) NOT NULL,
            oab VARCHAR(20) UNIQUE NOT NULL,
            telefone VARCHAR(20),
            email VARCHAR(255),
            especialidade VARCHAR(255)
        );

        CREATE TABLE Vara_Judicial (
            id_vara_judicial SERIAL PRIMARY KEY,
            nome_vara VARCHAR(255) NOT NULL,
            cidade VARCHAR(100),
            estado VARCHAR(50)
        );

        CREATE TABLE Status (
            id_status SERIAL PRIMARY KEY,
            descricao VARCHAR(255) NOT NULL,
            data_modificacao DATE
        );

        CREATE TABLE Categoria_caso (
            id_categoria_caso SERIAL PRIMARY KEY,
            descricao VARCHAR(255) NOT NULL,
            ativo BOOLEAN DEFAULT TRUE
        );

        CREATE TABLE Caso (
            id_caso SERIAL PRIMARY KEY,
            id_cliente INTEGER NOT NULL REFERENCES Cliente(id_cliente),
            id_advogado INTEGER NOT NULL REFERENCES Advogado(id_advogado),
            id_status INTEGER NOT NULL REFERENCES Status(id_status),
            id_vara_judicial INTEGER REFERENCES Vara_Judicial(id_vara_judicial),
            id_categoria_caso INTEGER REFERENCES Categoria_caso(id_categoria_caso),
            descricao TEXT,
            numero_processo VARCHAR(255) UNIQUE,
            data_fechamento DATE,
            data_abertura DATE NOT NULL
        );

        CREATE TABLE Andamento_processual (
            id_andamento SERIAL PRIMARY KEY,
            id_caso INTEGER NOT NULL REFERENCES Caso(id_caso),
            descricao TEXT,
            data_andamento DATE NOT NULL,
            responsavel VARCHAR(255)
        );

        CREATE TABLE Audiencia (
            id_audiencia SERIAL PRIMARY KEY,
            id_caso INTEGER NOT NULL REFERENCES Caso(id_caso),
            data_audiencia TIMESTAMP NOT NULL,
            horario TIME,
            descricao TEXT,
            endereco TEXT,
            tipo_audiencia VARCHAR(100)
        );

        CREATE TABLE Pecas (
            id_peca SERIAL PRIMARY KEY,
            id_caso INTEGER NOT NULL REFERENCES Caso(id_caso),
            descricao TEXT,
            data_registro DATE NOT NULL,
            tipo_midia VARCHAR(100)
        );

        CREATE TABLE Documento (
            id_documento SERIAL PRIMARY KEY,
            id_caso INTEGER NOT NULL REFERENCES Caso(id_caso),
            descricao TEXT,
            data_envio DATE,
            tipo VARCHAR(100),
            nome_arquivo VARCHAR(255)
        );

        CREATE TABLE Tarefa (
            id_tarefa SERIAL PRIMARY KEY,
            id_caso INTEGER NOT NULL REFERENCES Caso(id_caso),
            id_advogado INTEGER NOT NULL REFERENCES Advogado(id_advogado),
            descricao TEXT,
            data_tarefa DATE NOT NULL
        );
        ").await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})));
    }

    // Return a success message
    (StatusCode::OK, Json(json!({"message": "Database initialized successfully"})))
}


#[tuono_lib::api(PUT)]
async fn populate_db(_req: Request) -> impl IntoResponse {
    let mut client = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Database connection error: {}", e)})));
        }
    };

    let transaction = match client.transaction().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to start transaction: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to start transaction: {}", e)})));
        }
    };

    // --- Insert into independent tables first ---

    // 1. Cliente
    let client_ids = vec![
        (1, "João Silva", "joao.silva@example.com", "11987654321", "Rua A, 123", NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        (2, "Maria Souza", "maria.souza@example.com", "21998765432", "Avenida B, 456", NaiveDate::from_ymd_opt(2022, 5, 15).unwrap()),
        (3, "ABC Corp", "contato@abccorp.com", "31976543210", "Praça C, 789", NaiveDate::from_ymd_opt(2021, 11, 30).unwrap()),
        (4, "Carlos Lima", "carlos.lima@example.com", "41912345678", "Travessa D, 101", NaiveDate::from_ymd_opt(2024, 2, 20).unwrap()),
        (5, "DEF Ltda", "info@defltda.com", "51965432109", "Alameda E, 202", NaiveDate::from_ymd_opt(2023, 7, 10).unwrap()),
    ];
    for (id, nome, email, telefone, endereco, data_cadastro) in client_ids.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Cliente (id_cliente, nome, email, telefone, endereco, data_cadastro) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id_cliente) DO NOTHING;",
            &[id, nome, email, telefone, endereco, data_cadastro],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Cliente: {}", e)})));
        }
    }

    // 2. Pessoa_Fisica
    let pessoa_fisica_data = vec![
        (1, "111.111.111-11"),
        (2, "222.222.222-22"),
        (4, "444.444.444-44"),
    ];
    for (id_cliente, cpf) in pessoa_fisica_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Pessoa_Fisica (id_cliente, cpf) VALUES ($1, $2) ON CONFLICT (id_cliente) DO NOTHING;",
            &[id_cliente, cpf],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Pessoa_Fisica: {}", e)})));
        }
    }

    // 3. Pessoa_Juridica
    let pessoa_juridica_data = vec![
        (3, "00.000.000/0001-00"),
        (5, "11.111.111/0001-11"),
    ];
    for (id_cliente, cnpj) in pessoa_juridica_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Pessoa_Juridica (id_cliente, cnpj) VALUES ($1, $2) ON CONFLICT (id_cliente) DO NOTHING;",
            &[id_cliente, cnpj],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Pessoa_Juridica: {}", e)})));
        }
    }

    // 4. Advogado
    let advogado_ids = vec![
        (101, "Dr. Roberto Santos", "12345SP", "11987654322", "roberto.santos@adv.com", "Direito Civil"),
        (102, "Dra. Ana Costa", "67890RJ", "21998765433", "ana.costa@adv.com", "Direito Penal"),
    ];
    for (id, nome, oab, telefone, email, especialidade) in advogado_ids.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Advogado (id_advogado, nome, oab, telefone, email, especialidade) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id_advogado) DO NOTHING;",
            &[id, nome, oab, telefone, email, especialidade],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Advogado: {}", e)})));
        }
    }

    // 5. Vara_Judicial
    let vara_judicial_ids = vec![
        (201, "1ª Vara Cível de São Paulo", "São Paulo", "SP"),
        (202, "2ª Vara Criminal do Rio de Janeiro", "Rio de Janeiro", "RJ"),
    ];
    for (id, nome, cidade, estado) in vara_judicial_ids.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Vara_Judicial (id_vara_judicial, nome_vara, cidade, estado) VALUES ($1, $2, $3, $4) ON CONFLICT (id_vara_judicial) DO NOTHING;",
            &[id, nome, cidade, estado],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Vara_Judicial: {}", e)})));
        }
    }

    // 6. Status
    let status_ids = vec![
        (301, "Em Andamento", NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        (302, "Concluído", NaiveDate::from_ymd_opt(2024, 6, 20).unwrap()),
        (303, "Arquivado", NaiveDate::from_ymd_opt(2024, 6, 25).unwrap()),
    ];
    for (id, descricao, data) in status_ids.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Status (id_status, descricao, data_modificacao) VALUES ($1, $2, $3) ON CONFLICT (id_status) DO NOTHING;",
            &[id, descricao, data],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Status: {}", e)})));
        }
    }

    // 7. Categoria_caso
    let categoria_caso_ids = vec![
        (401, "Causa Trabalhista", true),
        (402, "Divórcio", true),
        (403, "Criminal - Homicídio", true),
    ];
    for (id, descricao, ativo) in categoria_caso_ids.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Categoria_caso (id_categoria_caso, descricao, ativo) VALUES ($1, $2, $3) ON CONFLICT (id_categoria_caso) DO NOTHING;",
            &[id, descricao, ativo],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Categoria_caso: {}", e)})));
        }
    }

    // --- Insert into dependent tables ---

    // 8. Caso
    let caso_data = vec![
        (1001, 1, 101, 301, Some(201), Some(401), "Reclamação trabalhista - Salários atrasados", "12345-67.2023.8.26.0001", None, NaiveDate::from_ymd_opt(2023, 3, 10).unwrap()),
        (1002, 2, 102, 302, Some(202), Some(402), "Ação de divórcio consensual", "87654-32.2022.8.19.0001", Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()), NaiveDate::from_ymd_opt(2022, 1, 20).unwrap()),
        (1003, 3, 101, 301, None, Some(403), "Defesa criminal - Homicídio culposo", "11223-44.2024.8.26.0002", None, NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
        (1004, 4, 101, 303, Some(201), Some(401), "Execução de título extrajudicial", "55667-88.2023.8.26.0003", Some(NaiveDate::from_ymd_opt(2024, 5, 1).unwrap()), NaiveDate::from_ymd_opt(2023, 9, 1).unwrap()),
        (1005, 5, 102, 301, Some(202), Some(402), "Recurso de apelação", "99887-66.2024.8.19.0002", None, NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()),
    ];
    for (id_caso, id_cliente, id_advogado, id_status, id_vara_judicial, id_categoria_caso, descricao, numero_processo, data_fechamento, data_abertura) in caso_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Caso (id_caso, id_cliente, id_advogado, id_status, id_vara_judicial, id_categoria_caso, descricao, numero_processo, data_fechamento, data_abertura) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) ON CONFLICT (id_caso) DO NOTHING;",
            &[id_caso, id_cliente, id_advogado, id_status, id_vara_judicial, id_categoria_caso, descricao, numero_processo, data_fechamento, data_abertura],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Caso: {}", e)})));
        }
    }

    // 9. Andamento_processual
    let andamento_processual_data = vec![
        (1, 1001, "Petição inicial protocolada", NaiveDate::from_ymd_opt(2023, 3, 15).unwrap(), "Dr. Roberto Santos"),
        (2, 1001, "Audiência de conciliação agendada", NaiveDate::from_ymd_opt(2023, 4, 1).unwrap(), "Secretaria"),
        (3, 1002, "Sentença proferida", NaiveDate::from_ymd_opt(2024, 6, 10).unwrap(), "Dra. Ana Costa"),
    ];
    for (id_andamento, id_caso, descricao, data_andamento, responsavel) in andamento_processual_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Andamento_processual (id_andamento, id_caso, descricao, data_andamento, responsavel) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id_andamento) DO NOTHING;",
            &[id_andamento, id_caso, descricao, data_andamento, responsavel],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Andamento_processual: {}", e)})));
        }
    }

    // 10. Audiencia
    let audiencia_data = vec![
        (1, 1001, NaiveDate::from_ymd_opt(2023, 4, 20).unwrap().and_time(NaiveTime::from_hms_opt(10, 0, 0).unwrap()), NaiveTime::from_hms_opt(10, 0, 0).unwrap(), "Conciliação", "Fórum Central, Sala 5", "Online"),
        (2, 1002, NaiveDate::from_ymd_opt(2022, 5, 5).unwrap().and_time(NaiveTime::from_hms_opt(14, 30, 0).unwrap()), NaiveTime::from_hms_opt(14, 30, 0).unwrap(), "Instrução e Julgamento", "Tribunal de Justiça, Sala 10", "Presencial"),
    ];
    for (id_audiencia, id_caso, data_audiencia, horario, descricao, endereco, tipo_audiencia) in audiencia_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Audiencia (id_audiencia, id_caso, data_audiencia, horario, descricao, endereco, tipo_audiencia) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (id_audiencia) DO NOTHING;",
            &[id_audiencia, id_caso, &data_audiencia, horario, descricao, endereco, tipo_audiencia], // Changed &data_audiencia to data_audiencia (already a Timestamp)
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Audiencia: {}", e)})));
        }
    }

    // 11. Pecas
    let pecas_data = vec![
        (1, 1001, "Contrato de trabalho", NaiveDate::from_ymd_opt(2023, 3, 10).unwrap(), "Documento"),
        (2, 1001, "Holerites (últimos 6 meses)", NaiveDate::from_ymd_opt(2023, 3, 10).unwrap(), "Documento"),
        (3, 1002, "Certidão de casamento", NaiveDate::from_ymd_opt(2022, 1, 15).unwrap(), "Documento"),
    ];
    for (id_peca, id_caso, descricao, data_registro, tipo_midia) in pecas_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Pecas (id_peca, id_caso, descricao, data_registro, tipo_midia) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id_peca) DO NOTHING;",
            &[id_peca, id_caso, descricao, data_registro, tipo_midia],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Pecas: {}", e)})));
        }
    }

    // 12. Documento
    let documento_data = vec![
        (1, 1001, "Comprovante de residência do cliente", Some(NaiveDate::from_ymd_opt(2023, 3, 8).unwrap()), "PDF", "comprovante_joao.pdf"),
        (2, 1001, "Procuração assinada", Some(NaiveDate::from_ymd_opt(2023, 3, 9).unwrap()), "PDF", "procuracao_joao.pdf"),
        (3, 1002, "Documentos pessoais da cliente", Some(NaiveDate::from_ymd_opt(2022, 1, 10).unwrap()), "JPG", "docs_maria.zip"),
    ];
    for (id_documento, id_caso, descricao, data_envio, tipo, nome_arquivo) in documento_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Documento (id_documento, id_caso, descricao, data_envio, tipo, nome_arquivo) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id_documento) DO NOTHING;",
            &[id_documento, id_caso, descricao, data_envio, tipo, nome_arquivo],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Documento: {}", e)})));
        }
    }

    // 13. Tarefa
    let tarefa_data = vec![
        (1, 1001, 101, "Analisar documentação", NaiveDate::from_ymd_opt(2023, 3, 12).unwrap()),
        (2, 1001, 101, "Preparar defesa", NaiveDate::from_ymd_opt(2023, 3, 25).unwrap()),
        (3, 1002, 102, "Revisar petição", NaiveDate::from_ymd_opt(2022, 1, 18).unwrap()),
    ];
    for (id_tarefa, id_caso, id_advogado, descricao, data_tarefa) in tarefa_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Tarefa (id_tarefa, id_caso, id_advogado, descricao, data_tarefa) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id_tarefa) DO NOTHING;",
            &[id_tarefa, id_caso, id_advogado, descricao, data_tarefa],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Tarefa: {}", e)})));
        }
    }

    match transaction.commit().await {
        Ok(_) => {
            // --- Atualizar a sequência do SERIAL após as inserções manuais ---
            let max_cliente_id_query = "SELECT setval('cliente_id_cliente_seq', (SELECT MAX(id_cliente) FROM Cliente), TRUE);";
            if let Err(e) = client.execute(max_cliente_id_query, &[]).await {
                eprintln!("Failed to update cliente_id_cliente_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update sequence: {}", e)})));
            }

            (StatusCode::CREATED, Json(json!({"message": "Database populated with fictitious data successfully."})))
        },
        Err(e) => {
            eprintln!("Failed to commit transaction: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to commit transaction: {}", e)})))
        }
    }
}