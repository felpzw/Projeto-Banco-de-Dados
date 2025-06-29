use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::StatusCode;
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
            arquivo BYTEA,
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

    (StatusCode::OK, Json(json!({"message": "Database initialized successfully"})))
}


#[tuono_lib::api(PUT)]
async fn populate_db(_req: Request) -> impl IntoResponse {
    let mut client = match connect_db().await {
        Ok(client) => client,
        Err(e) => {
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

    // 1. Cliente (Bem mais entradas e diversidade)
    let clients_data = vec![
        // PF
        (1, "João Silva", "joao.silva@example.com", "11987654321", "Rua A, 123, Florianópolis", NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), Some("111.111.111-11"), None),
        (2, "Maria Souza", "maria.souza@example.com", "21998765432", "Avenida B, 456, Joinville", NaiveDate::from_ymd_opt(2022, 5, 15).unwrap(), Some("222.222.222-22"), None),
        (3, "Pedro Almeida", "pedro.almeida@example.com", "48991234567", "Rua C, 789, Blumenau", NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(), Some("333.333.333-33"), None),
        (4, "Ana Santos", "ana.santos@example.com", "48988765432", "Travessa D, 101, Chapecó", NaiveDate::from_ymd_opt(2023, 11, 5).unwrap(), Some("444.444.444-44"), None),
        (5, "Lucas Ferreira", "lucas.ferreira@example.com", "48992345678", "Av. E, 202, Lages", NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(), Some("555.555.555-55"), None),
        (6, "Mariana Costa", "mariana.costa@example.com", "48993456789", "Rua F, 303, Criciúma", NaiveDate::from_ymd_opt(2023, 7, 1).unwrap(), Some("666.666.666-66"), None),
        (11, "Roberto Pereira", "roberto.p@example.com", "48994567890", "Av. do Contorno, 50, Palhoça", NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(), Some("777.777.777-77"), None),
        (12, "Fernanda Lima", "fernanda.l@example.com", "48991122334", "Rua das Palmeiras, 10, Itajaí", NaiveDate::from_ymd_opt(2023, 9, 15).unwrap(), Some("888.888.888-88"), None),

        // PJ
        (7, "Tech Solutions Ltda", "contato@techsol.com", "4832109876", "Rua G, 404, São José", NaiveDate::from_ymd_opt(2022, 9, 1).unwrap(), None, Some("00.000.000/0001-00")),
        (8, "Construtora Alfa", "contato@alfa.com", "4833210987", "Av. H, 505, Palhoça", NaiveDate::from_ymd_opt(2023, 3, 20).unwrap(), None, Some("11.111.111/0001-11")),
        (9, "Serviços Beta S.A.", "info@beta.com", "4834321098", "Rua I, 606, Itajaí", NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(), None, Some("22.222.222/0001-22")),
        (10, "Distribuidora Gama", "vendas@gama.com", "4835432109", "Rod. J, 707, Tubarão", NaiveDate::from_ymd_opt(2023, 10, 15).unwrap(), None, Some("33.333.333/0001-33")),
        (13, "Logística Delta EIRELI", "contato@delta.com", "4836543210", "Av. Principal, 1000, Lages", NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(), None, Some("44.444.444/0001-44")),
        (14, "Consultoria Epsilon", "rh@epsilon.com", "4837654321", "Praça Central, 25, Criciúma", NaiveDate::from_ymd_opt(2023, 6, 10).unwrap(), None, Some("55.555.555/0001-55")),
    ];

    for (id, nome, email, telefone, endereco, data_cadastro, cpf, cnpj) in clients_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Cliente (id_cliente, nome, email, telefone, endereco, data_cadastro) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id_cliente) DO UPDATE SET nome = EXCLUDED.nome, email = EXCLUDED.email, telefone = EXCLUDED.telefone, endereco = EXCLUDED.endereco, data_cadastro = EXCLUDED.data_cadastro;",
            &[id, nome, email, telefone, endereco, data_cadastro],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Cliente: {}", e)})));
        }

        if let Some(c) = cpf {
            if let Err(e) = transaction.execute(
                "INSERT INTO Pessoa_Fisica (id_cliente, cpf) VALUES ($1, $2) ON CONFLICT (id_cliente) DO UPDATE SET cpf = EXCLUDED.cpf;",
                &[id, c],
            ).await {
                eprintln!("Failed to insert Pessoa_Fisica: {}", e);
                let _ = transaction.rollback().await;
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert physical person details: {}", e)})));
            }
        }
        if let Some(c) = cnpj {
            if let Err(e) = transaction.execute(
                "INSERT INTO Pessoa_Juridica (id_cliente, cnpj) VALUES ($1, $2) ON CONFLICT (id_cliente) DO UPDATE SET cnpj = EXCLUDED.cnpj;",
                &[id, c],
            ).await {
                eprintln!("Failed to insert Pessoa_Juridica: {}", e);
                let _ = transaction.rollback().await;
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert legal person details: {}", e)})));
            }
        }
    }


    // 2. Advogado (Mais entradas)
    let advogados_data = vec![
        (101, "Dr. Roberto Santos", "12345SC", "48999001122", "roberto.santos@adv.com", "Direito Civil"),
        (102, "Dra. Ana Costa", "67890SC", "48998002233", "ana.costa@adv.com", "Direito Penal"),
        (103, "Dr. Gabriel Mendes", "11223SC", "48997003344", "gabriel.mendes@adv.com", "Direito Trabalhista"),
        (104, "Dra. Laura Ribeiro", "44556SC", "48996004455", "laura.ribeiro@adv.com", "Direito Empresarial"),
        (105, "Dr. Felipe Castro", "77889SC", "48995005566", "felipe.castro@adv.com", "Direito Tributário"),
        (106, "Dra. Sofia Lima", "22334SC", "48994006677", "sofia.lima@adv.com", "Direito Imobiliário"),
        (107, "Dr. André Pereira", "55667SC", "48993007788", "andre.pereira@adv.com", "Direito do Consumidor"),
    ];
    for (id, nome, oab, telefone, email, especialidade) in advogados_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Advogado (id_advogado, nome, oab, telefone, email, especialidade) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id_advogado) DO UPDATE SET nome = EXCLUDED.nome, oab = EXCLUDED.oab, telefone = EXCLUDED.telefone, email = EXCLUDED.email, especialidade = EXCLUDED.especialidade;",
            &[id, nome, oab, telefone, email, especialidade],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Advogado: {}", e)})));
        }
    }

    // 3. Vara_Judicial (Mais entradas, todas em SC)
    let varas_judiciais_data = vec![
        (201, "1ª Vara Cível", "Florianópolis", "SC"),
        (202, "2ª Vara Criminal", "Florianópolis", "SC"),
        (203, "Vara do Trabalho", "São José", "SC"),
        (204, "Vara de Família", "Joinville", "SC"),
        (205, "Vara da Fazenda Pública", "Blumenau", "SC"),
        (206, "3ª Vara Cível", "Chapecó", "SC"),
        (207, "Vara de Execuções Fiscais", "Lages", "SC"),
        (208, "Juizado Especial Cível", "Criciúma", "SC"),
    ];
    for (id, nome_vara, cidade, estado) in varas_judiciais_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Vara_Judicial (id_vara_judicial, nome_vara, cidade, estado) VALUES ($1, $2, $3, $4) ON CONFLICT (id_vara_judicial) DO UPDATE SET nome_vara = EXCLUDED.nome_vara, cidade = EXCLUDED.cidade, estado = EXCLUDED.estado;",
            &[id, nome_vara, cidade, estado],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Vara_Judicial: {}", e)})));
        }
    }

    // 4. Status (Mais entradas)
    let status_data = vec![
        (301, "Em Andamento", NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        (302, "Concluído", NaiveDate::from_ymd_opt(2024, 6, 20).unwrap()),
        (303, "Arquivado", NaiveDate::from_ymd_opt(2024, 6, 25).unwrap()),
        (304, "Pendente", NaiveDate::from_ymd_opt(2024, 1, 10).unwrap()),
        (305, "Suspenso", NaiveDate::from_ymd_opt(2024, 3, 5).unwrap()),
        (306, "Recurso", NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()),
        (307, "Em Julgamento", NaiveDate::from_ymd_opt(2024, 5, 15).unwrap()),
    ];
    for (id, descricao, data) in status_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Status (id_status, descricao, data_modificacao) VALUES ($1, $2, $3) ON CONFLICT (id_status) DO UPDATE SET descricao = EXCLUDED.descricao, data_modificacao = EXCLUDED.data_modificacao;",
            &[id, descricao, data],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Status: {}", e)})));
        }
    }

    // 5. Categoria_caso (Mais entradas)
    let categoria_caso_data = vec![
        (401, "Cível - Cobrança", true),
        (402, "Criminal - Furto", true),
        (403, "Trabalhista - Rescisão Indireta", true),
        (404, "Família - Divórcio", true),
        (405, "Tributário - Restituição de Imposto", true),
        (406, "Ambiental - Licenciamento", true),
        (407, "Consumidor - Vício de Produto", true),
        (408, "Administrativo - Concurso Público", true),
    ];
    for (id, descricao, ativo) in categoria_caso_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Categoria_caso (id_categoria_caso, descricao, ativo) VALUES ($1, $2, $3) ON CONFLICT (id_categoria_caso) DO UPDATE SET descricao = EXCLUDED.descricao, ativo = EXCLUDED.ativo;",
            &[id, descricao, ativo],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Categoria_caso: {}", e)})));
        }
    }

    // 6. Caso (MUITO MAIS ENTRADAS, sem IDs fixos, capturando os IDs gerados)
    // Usamos um template mais longo e aleatório para clientes/advogados/status/varas/categorias
    let cases_data_template = vec![
        // Combinando Clientes (1-14), Advogados (101-107), Status (301-307), Varas (201-208), Categorias (401-408)
        // Cliente, Advogado, Status, Vara Judicial (Opt), Categoria (Opt), Descricao (Opt), Num Processo (Opt), Data Fechamento (Opt), Data Abertura
        (1, 101, 301, Some(201), Some(401), Some("Ação de Cobrança de Dívida"), Some("12345-67.2023.8.24.0001"), None, NaiveDate::from_ymd_opt(2023, 3, 10).unwrap()),
        (2, 102, 304, Some(204), Some(404), Some("Processo de Divórcio Litigioso"), Some("98765-43.2024.8.24.0003"), None, NaiveDate::from_ymd_opt(2024, 1, 20).unwrap()),
        (3, 103, 306, Some(203), Some(403), Some("Recurso em Reclamatória Trabalhista"), Some("11223-44.2023.8.24.0002"), None, NaiveDate::from_ymd_opt(2023, 7, 5).unwrap()),
        (4, 104, 302, Some(205), Some(405), Some("Processo de Restituição de ICMS"), Some("55667-88.2024.8.24.0004"), Some(NaiveDate::from_ymd_opt(2024, 6, 25).unwrap()), NaiveDate::from_ymd_opt(2024, 2, 15).unwrap()),
        (5, 105, 303, Some(206), Some(406), Some("Processo de Regularização Ambiental"), Some("99887-66.2023.8.24.0005"), None, NaiveDate::from_ymd_opt(2023, 9, 1).unwrap()),
        (6, 106, 301, Some(207), Some(407), Some("Ação de Indenização por Vício de Produto"), Some("12398-76.2024.8.24.0006"), None, NaiveDate::from_ymd_opt(2024, 3, 12).unwrap()),
        (7, 107, 304, Some(208), Some(408), Some("Mandado de Segurança - Concurso Público"), Some("45678-90.2023.8.24.0007"), None, NaiveDate::from_ymd_opt(2023, 11, 20).unwrap()),
        (8, 101, 305, Some(201), Some(401), Some("Ação de Execução Hipotecária"), Some("11335-55.2024.8.24.0001"), None, NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()),
        (9, 102, 301, Some(202), Some(402), Some("Ação Penal Pública Condicionada"), Some("22446-66.2023.8.24.0002"), None, NaiveDate::from_ymd_opt(2023, 5, 30).unwrap()),
        (10, 103, 302, Some(203), Some(403), Some("Dissídio Coletivo"), Some("33557-77.2024.8.24.0003"), Some(NaiveDate::from_ymd_opt(2024, 6, 10).unwrap()), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        (11, 104, 304, Some(204), Some(404), Some("Guarda Compartilhada de Menor"), Some("44668-88.2023.8.24.0004"), None, NaiveDate::from_ymd_opt(2023, 8, 15).unwrap()),
        (12, 105, 306, Some(205), Some(405), Some("Revisão Tributária de ITBI"), Some("55779-99.2024.8.24.0005"), None, NaiveDate::from_ymd_opt(2024, 3, 5).unwrap()),
        (13, 106, 307, Some(206), Some(406), Some("Ação de Demarcação de Terras"), Some("66880-00.2023.8.24.0006"), None, NaiveDate::from_ymd_opt(2023, 10, 1).unwrap()),
        (14, 107, 301, Some(207), Some(407), Some("Contestação em Ação de Consumidor"), Some("77991-11.2024.8.24.0007"), None, NaiveDate::from_ymd_opt(2024, 5, 20).unwrap()),
        (1, 103, 302, Some(208), Some(408), Some("Recurso Administrativo"), Some("88112-22.2023.8.24.0008"), Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()), NaiveDate::from_ymd_opt(2023, 2, 10).unwrap()),
        (2, 104, 303, Some(201), Some(401), Some("Cobrança de Aluguéis Atrasados"), Some("99223-33.2024.8.24.0001"), None, NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
        (3, 105, 304, Some(202), Some(402), Some("Habeas Corpus"), Some("00334-44.2023.8.24.0002"), None, NaiveDate::from_ymd_opt(2023, 7, 20).unwrap()),
        (4, 106, 301, Some(203), Some(403), Some("Mandado de Segurança Trabalhista"), Some("11445-55.2024.8.24.0003"), None, NaiveDate::from_ymd_opt(2024, 4, 10).unwrap()),
        (5, 107, 305, Some(204), Some(404), Some("Regulamentação de Visitas"), Some("22556-66.2023.8.24.0004"), None, NaiveDate::from_ymd_opt(2023, 9, 5).unwrap()),
        (6, 101, 306, Some(205), Some(405), Some("Defesa em Execução Fiscal"), Some("33667-77.2024.8.24.0005"), None, NaiveDate::from_ymd_opt(2024, 2, 20).unwrap()),
    ];

    let insert_caso_query = "
        INSERT INTO Caso (id_cliente, id_advogado, id_status, id_vara_judicial, id_categoria_caso, descricao, numero_processo, data_abertura, data_fechamento)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id_caso;
    ";

    let mut generated_case_ids: Vec<i32> = Vec::new(); // Para capturar IDs de casos gerados

    for (id_cliente, id_advogado, id_status, id_vara_judicial, id_categoria_caso, descricao, numero_processo, data_fechamento, data_abertura) in cases_data_template.iter() {
        let rows = match transaction.query(
            insert_caso_query,
            &[id_cliente, id_advogado, id_status, id_vara_judicial, id_categoria_caso, descricao, numero_processo, data_abertura, data_fechamento],
        ).await {
            Ok(rows) => rows,
            Err(e) => {
                eprintln!("Failed to insert Caso: {}", e);
                let _ = transaction.rollback().await;
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to create case: {}", e)})));
            }
        };
        let new_id_caso: i32 = rows[0].get("id_caso");
        generated_case_ids.push(new_id_caso); // Captura os IDs gerados para uso posterior
    }


    // 7. Andamento_processual, Audiencia, Pecas, Documento, Tarefa (POUCAS ENTRADAS)
    // Usar apenas os 3 primeiros IDs de casos gerados para vincular os andamentos, etc.
    let case_id_dep1 = generated_case_ids.get(0).cloned().unwrap_or(1); // Ex: Caso 1
    let case_id_dep2 = generated_case_ids.get(1).cloned().unwrap_or(2); // Ex: Caso 2
    let case_id_dep3 = generated_case_ids.get(2).cloned().unwrap_or(3); // Ex: Caso 3
    let adv_id_dep1 = advogados_data.get(0).map(|a| a.0).unwrap_or(101); // Dr. Roberto
    let adv_id_dep2 = advogados_data.get(1).map(|a| a.0).unwrap_or(102); // Dra. Ana

    // Andamento_processual (Apenas 3 entradas)
    let andamento_processual_data = vec![
        (1, case_id_dep1, "Petição inicial protocolada", NaiveDate::from_ymd_opt(2023, 3, 15).unwrap(), "Dr. Roberto Santos"),
        (2, case_id_dep2, "Audiência de conciliação agendada", NaiveDate::from_ymd_opt(2023, 4, 1).unwrap(), "Secretaria"),
        (3, case_id_dep3, "Sentença proferida", NaiveDate::from_ymd_opt(2024, 6, 10).unwrap(), "Dra. Ana Costa"),
    ];
    for (id_andamento, id_caso, descricao, data_andamento, responsavel) in andamento_processual_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Andamento_processual (id_andamento, id_caso, descricao, data_andamento, responsavel) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id_andamento) DO UPDATE SET id_caso = EXCLUDED.id_caso, descricao = EXCLUDED.descricao, data_andamento = EXCLUDED.data_andamento, responsavel = EXCLUDED.responsavel;",
            &[id_andamento, id_caso, descricao, data_andamento, responsavel],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Andamento_processual: {}", e)})));
        }
    }

    // Audiencia (Apenas 2 entradas)
    let audiencia_data = vec![
        (1, case_id_dep1, NaiveDate::from_ymd_opt(2023, 4, 20).unwrap().and_time(NaiveTime::from_hms_opt(10, 0, 0).unwrap()), NaiveTime::from_hms_opt(10, 0, 0).unwrap(), "Conciliação", "Fórum Central, Sala 5", "Online"),
        (2, case_id_dep2, NaiveDate::from_ymd_opt(2022, 5, 5).unwrap().and_time(NaiveTime::from_hms_opt(14, 30, 0).unwrap()), NaiveTime::from_hms_opt(14, 30, 0).unwrap(), "Instrução e Julgamento", "Tribunal de Justiça, Sala 10", "Presencial"),
    ];
    for (id_audiencia, id_caso, data_audiencia, horario, descricao, endereco, tipo_audiencia) in audiencia_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Audiencia (id_audiencia, id_caso, data_audiencia, horario, descricao, endereco, tipo_audiencia) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (id_audiencia) DO UPDATE SET id_caso = EXCLUDED.id_caso, data_audiencia = EXCLUDED.data_audiencia, horario = EXCLUDED.horario, descricao = EXCLUDED.descricao, endereco = EXCLUDED.endereco, tipo_audiencia = EXCLUDED.tipo_audiencia;",
            &[id_audiencia, id_caso, &data_audiencia, horario, descricao, endereco, tipo_audiencia],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Audiencia: {}", e)})));
        }
    }

    // Pecas (Apenas 3 entradas)
    let pecas_data = vec![
        (1, case_id_dep1, "Contrato de trabalho", NaiveDate::from_ymd_opt(2023, 3, 10).unwrap(), "Documento"),
        (2, case_id_dep2, "Holerites (últimos 6 meses)", NaiveDate::from_ymd_opt(2023, 3, 10).unwrap(), "Documento"),
        (3, case_id_dep3, "Certidão de casamento", NaiveDate::from_ymd_opt(2022, 1, 15).unwrap(), "Documento"),
    ];
    for (id_peca, id_caso, descricao, data_registro, tipo_midia) in pecas_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Pecas (id_peca, id_caso, descricao, data_registro, tipo_midia) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id_peca) DO UPDATE SET id_caso = EXCLUDED.id_caso, descricao = EXCLUDED.descricao, data_registro = EXCLUDED.data_registro, tipo_midia = EXCLUDED.tipo_midia;",
            &[id_peca, id_caso, descricao, data_registro, tipo_midia],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Pecas: {}", e)})));
        }
    }

    // Documento (Apenas 3 entradas, vinculadas aos primeiros casos, arquivo BYTEA NULL)
    let documento_data = vec![
        (1, case_id_dep1, "Comprovante de residência", Some(NaiveDate::from_ymd_opt(2023, 3, 8).unwrap()), "comprovante_joao.pdf"),
        (2, case_id_dep2, "Procuração assinada", Some(NaiveDate::from_ymd_opt(2023, 3, 9).unwrap()), "procuracao_ana.pdf"),
        (3, case_id_dep3, "Contrato de Prestação", Some(NaiveDate::from_ymd_opt(2024, 1, 10).unwrap()), "contrato_tech.pdf"),
    ];
    for (id_documento, id_caso, descricao, data_envio, nome_arquivo) in documento_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Documento (id_documento, id_caso, descricao, data_envio, nome_arquivo, arquivo) VALUES ($1, $2, $3, $4, $5, NULL) ON CONFLICT (id_documento) DO UPDATE SET id_caso = EXCLUDED.id_caso, descricao = EXCLUDED.descricao, data_envio = EXCLUDED.data_envio, nome_arquivo = EXCLUDED.nome_arquivo, arquivo = EXCLUDED.arquivo;",
            &[id_documento, id_caso, descricao, data_envio, nome_arquivo],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Documento: {}", e)})));
        }
    }

    // Tarefa (Apenas 3 entradas)
    let tarefa_data = vec![
        (1, case_id_dep1, adv_id_dep1, "Analisar documentação", NaiveDate::from_ymd_opt(2023, 3, 12).unwrap()),
        (2, case_id_dep2, adv_id_dep2, "Preparar defesa", NaiveDate::from_ymd_opt(2023, 3, 25).unwrap()),
        (3, case_id_dep3, adv_id_dep1, "Revisar petição", NaiveDate::from_ymd_opt(2022, 1, 18).unwrap()),
    ];
    for (id_tarefa, id_caso, id_advogado, descricao, data_tarefa) in tarefa_data.iter() {
        if let Err(e) = transaction.execute(
            "INSERT INTO Tarefa (id_tarefa, id_caso, id_advogado, descricao, data_tarefa) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id_tarefa) DO UPDATE SET id_caso = EXCLUDED.id_caso, id_advogado = EXCLUDED.id_advogado, descricao = EXCLUDED.descricao, data_tarefa = EXCLUDED.data_tarefa;",
            &[id_tarefa, id_caso, id_advogado, descricao, data_tarefa],
        ).await {
            let _ = transaction.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to insert Tarefa: {}", e)})));
        }
    }

    match transaction.commit().await {
        Ok(_) => {
            // Atualizar TODAS as sequências após inserções
            let max_cliente_id_query = "SELECT setval('cliente_id_cliente_seq', (SELECT MAX(id_cliente) FROM Cliente), TRUE);";
            if let Err(e) = client.execute(max_cliente_id_query, &[]).await {
                eprintln!("Failed to update cliente_id_cliente_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update sequence: {}", e)})));
            }
            let max_documento_id_query = "SELECT setval('documento_id_documento_seq', (SELECT MAX(id_documento) FROM Documento), TRUE);";
            if let Err(e) = client.execute(max_documento_id_query, &[]).await {
                eprintln!("Failed to update documento_id_documento_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update documento sequence: {}", e)})));
            }
            let max_advogado_id_query = "SELECT setval('advogado_id_advogado_seq', (SELECT MAX(id_advogado) FROM Advogado), TRUE);";
            if let Err(e) = client.execute(max_advogado_id_query, &[]).await {
                eprintln!("Failed to update advogado_id_advogado_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update advogado sequence: {}", e)})));
            }
            let max_vara_judicial_id_query = "SELECT setval('vara_judicial_id_vara_judicial_seq', (SELECT MAX(id_vara_judicial) FROM Vara_Judicial), TRUE);";
            if let Err(e) = client.execute(max_vara_judicial_id_query, &[]).await {
                eprintln!("Failed to update vara_judicial_id_vara_judicial_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update vara_judicial sequence: {}", e)})));
            }
            let max_status_id_query = "SELECT setval('status_id_status_seq', (SELECT MAX(id_status) FROM Status), TRUE);";
            if let Err(e) = client.execute(max_status_id_query, &[]).await {
                eprintln!("Failed to update status_id_status_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update status sequence: {}", e)})));
            }
            let max_categoria_caso_id_query = "SELECT setval('categoria_caso_id_categoria_caso_seq', (SELECT MAX(id_categoria_caso) FROM Categoria_caso), TRUE);";
            if let Err(e) = client.execute(max_categoria_caso_id_query, &[]).await {
                eprintln!("Failed to update categoria_caso_id_categoria_caso_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update categoria_caso sequence: {}", e)})));
            }
            let max_caso_id_query = "SELECT setval('caso_id_caso_seq', (SELECT MAX(id_caso) FROM Caso), TRUE);";
            if let Err(e) = client.execute(max_caso_id_query, &[]).await {
                eprintln!("Failed to update caso_id_caso_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update caso sequence: {}", e)})));
            }
            // Sequências de tabelas dependentes
            let max_andamento_id_query = "SELECT setval('andamento_processual_id_andamento_seq', (SELECT MAX(id_andamento) FROM Andamento_processual), TRUE);";
            if let Err(e) = client.execute(max_andamento_id_query, &[]).await {
                eprintln!("Failed to update andamento_processual_id_andamento_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update andamento_processual sequence: {}", e)})));
            }
            let max_audiencia_id_query = "SELECT setval('audiencia_id_audiencia_seq', (SELECT MAX(id_audiencia) FROM Audiencia), TRUE);";
            if let Err(e) = client.execute(max_audiencia_id_query, &[]).await {
                eprintln!("Failed to update audiencia_id_audiencia_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update audiencia sequence: {}", e)})));
            }
            let max_pecas_id_query = "SELECT setval('pecas_id_peca_seq', (SELECT MAX(id_peca) FROM Pecas), TRUE);";
            if let Err(e) = client.execute(max_pecas_id_query, &[]).await {
                eprintln!("Failed to update pecas_id_peca_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update pecas sequence: {}", e)})));
            }
            let max_documento_id_query = "SELECT setval('documento_id_documento_seq', (SELECT MAX(id_documento) FROM Documento), TRUE);";
            if let Err(e) = client.execute(max_documento_id_query, &[]).await {
                eprintln!("Failed to update documento_id_documento_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update documento sequence: {}", e)})));
            }
            let max_tarefa_id_query = "SELECT setval('tarefa_id_tarefa_seq', (SELECT MAX(id_tarefa) FROM Tarefa), TRUE);";
            if let Err(e) = client.execute(max_tarefa_id_query, &[]).await {
                eprintln!("Failed to update tarefa_id_tarefa_seq: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update tarefa sequence: {}", e)})));
            }

            (StatusCode::CREATED, Json(json!({"message": "Database populated with fictitious data successfully."})))
        },
        Err(e) => {
            eprintln!("Failed to commit transaction: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to commit transaction: {}", e)})))
        }
    }
}