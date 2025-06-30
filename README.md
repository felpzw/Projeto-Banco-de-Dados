# 📚 Jurídico IA: Plataforma de Gestão Jurídica com IA Integrada

Bem-vindo ao **Jurídico IA**, uma plataforma inovadora desenvolvida para auxiliar profissionais e estudantes do direito. Este projeto utiliza o framework web Rust **Tuono** no backend, **React/Next.js** no frontend e integra a **Inteligência Artificial Ollama** para otimizar diversas tarefas jurídicas.

## 🚀 Funcionalidades Principais

* **Gestão de Clientes:** Cadastro, visualização, edição e exclusão de clientes (Pessoa Física e Jurídica).
* **Gestão de Documentos:** Upload, visualização, download, edição e exclusão de documentos (especialmente PDFs) vinculados a casos.
* **Gestão de Casos Jurídicos:** Cadastro detalhado de casos, com vinculação a clientes, advogados, status, varas e categorias, exibindo informações completas.
* **IA Integrada (Ollama):** Faça perguntas sobre o conteúdo de seus documentos PDFs, utilizando modelos de Linguagem Grande (LLM) rodando localmente com Ollama para obter respostas contextuais.
* **Relatórios Visuais:** Geração de relatórios que relacionam dados de clientes, casos e documentos, apresentados em gráficos para insights rápidos.
* **Ferramentas de Desenvolvimento:** Página de configurações para gerenciar o estado do banco de dados (limpar, iniciar, popular) de forma fácil durante o desenvolvimento.

## ⚙️ Como Iniciar o Projeto

Este projeto utiliza **Docker** para gerenciar o banco de dados PostgreSQL e a instância do Ollama, simplificando o setup do ambiente.

### **Pré-requisitos**

* Docker e Docker Compose instalados.
* Rust e Cargo instalados (para o desenvolvimento do backend).
* Node.js e npm/yarn instalados (para o desenvolvimento do frontend).

### **Passo a Passo**

1.  **Clone o Repositório:**
    ```bash
    git clone [URL_DO_SEU_REPOSITORIO]
    cd Projeto-Banco-de-Dados
    ```

2.  **Configurar Variáveis de Ambiente:**
    Crie um arquivo chamado `var.env` na raiz do projeto (mesmo diretório do `Cargo.toml` e `docker-compose.yml`).
    Insira as seguintes linhas, ajustando conforme necessário (especialmente `OLLAMA_API_URL` se seu Ollama não for localhost):

    ```
    DATABASE_URL=host=localhost port=5432 user=usuario password=1234 dbname=banco_de_dados
    OLLAMA_API_URL=http://localhost:11434/
    ```
    **Observação sobre `OLLAMA_API_URL`:** Se o seu Ollama estiver rodando em um servidor diferente ou via um proxy, use a URL base desse servidor (ex: `https://ollama.vlab.ufsc.br/`).

3.  **Subir o Banco de Dados e o Ollama com Docker Compose:**
    Este comando iniciará os contêineres do PostgreSQL e do Ollama em segundo plano.
    ```bash
    docker-compose up -d
    ```
    Aguarde alguns instantes para que o PostgreSQL e o Ollama inicializem completamente.

4.  **Instalar Modelos do Ollama (Manual):**
    Você precisa baixar os modelos que deseja usar no Ollama. Acesse a interface do Ollama (geralmente `http://localhost:11434` ou a URL configurada) ou use o comando `ollama run` para baixar um modelo.
    Exemplo para baixar o Llama2 (ou outro modelo de sua escolha):
    ```bash
    docker exec -it ollama ollama run llama2
    # Você pode sair digitando /bye ou ctrl+d
    ```
    Repita para outros modelos como `qwen2.5:7b` ou `mistral-large:123b` se desejar usá-los. Certifique-se de que os modelos referenciados no frontend (`qwen2.5:7b` no exemplo do `curl` de teste) estejam instalados.

5.  **Iniciar o Projeto Tuono:**
    Este comando compilará o backend Rust e iniciará o servidor de desenvolvimento Tuono/React.
    ```bash
    tuono dev
    ```
    O projeto estará acessível em `http://localhost:3000` no seu navegador.

## 🛠️ Ferramentas de Desenvolvimento (Página de Configurações)

Acesse `http://localhost:3000/configuracoes` para gerenciar o estado do seu banco de dados durante o desenvolvimento:

* **LIMPAR DB (DEBUG):** Executa `DELETE /api/clean`. **Cuidado:** Apaga **TODOS** os dados das tabelas.
* **INICIAR DB (DEBUG):** Executa `POST /api/init`. Cria a estrutura de tabelas no banco de dados.
* **POPULAR DB (DEBUG):** Executa `PUT /api/populate_db`. Insere dados fictícios (clientes, advogados, casos, etc.) no banco, atualizando os existentes se houver conflito.

**Fluxo Recomendado para Teste/Desenvolvimento:**
1.  `LIMPAR DB (DEBUG)`
2.  `INICIAR DB (DEBUG)`
3.  `POPULAR DB (DEBUG)`

Isso garantirá que seu banco de dados esteja sempre em um estado consistente para testes.

---
