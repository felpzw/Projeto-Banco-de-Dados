import { useState, useEffect, useCallback } from 'react';
import type { JSX } from 'react';
import { useRouter } from 'tuono';
import type { TuonoRouteProps } from 'tuono';

// Interfaces para os dados recebidos via props (pré-renderizados)
interface OllamaModelProps {
  id: string;
  nome: string;
}

interface DocumentProps {
  id_documento: number;
  nome_arquivo: string;
}

interface IaIntegratedPageData {
  ollama_models: OllamaModelProps[];
  documents: DocumentProps[];
}

export default function IaIntegratedPage({ data, isLoading: propIsLoading }: TuonoRouteProps<IaIntegratedPageData>): JSX.Element {
  const router = useRouter();
  const [ollamaModels, setOllamaModels] = useState<OllamaModelProps[]>(data?.ollama_models || []);
  const [documents, setDocuments] = useState<DocumentProps[]>(data?.documents || []);
  const [selectedModel, setSelectedModel] = useState<string>('');
  const [selectedDocumentName, setSelectedDocumentName] = useState<string>('');
  const [question, setQuestion] = useState<string>('');
  const [llmResponse, setLlmResponse] = useState<string>('');
  const [isLoadingResponse, setIsLoadingResponse] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isLoadingPage, setIsLoadingPage] = useState(propIsLoading);

  useEffect(() => {
    const initializeDropdowns = (models: OllamaModelProps[], docs: DocumentProps[]) => {
      if (models.length > 0) {
        setSelectedModel(models[0].id);
      } else {
        setSelectedModel(''); // Garante que esteja vazio se não houver modelos
      }

      if (docs.length > 0) {
        setSelectedDocumentName(docs[0].nome_arquivo);
      } else {
        setSelectedDocumentName(''); // Garante que esteja vazio se não houver documentos
      }
    };

    const fetchDataClient = async () => {
      setIsLoadingPage(true);
      try {
        const modelsRes = await fetch('/api/ollama');
        if (!modelsRes.ok) {
            const errorBody = await modelsRes.text();
            throw new Error(`Falha ao carregar modelos Ollama: ${modelsRes.status} - ${errorBody}`);
        }
        const modelsData: OllamaModelProps[] = await modelsRes.json();
        setOllamaModels(modelsData);

        const docsRes = await fetch('/api/documentos');
        if (!docsRes.ok) {
            const errorBody = await docsRes.text();
            throw new Error(`Falha ao carregar documentos: ${docsRes.status} - ${errorBody}`);
        }
        const docsData: DocumentProps[] = await docsRes.json();
        setDocuments(docsData);

        initializeDropdowns(modelsData, docsData); // Inicializa os dropdowns após o fetch

      } catch (err: any) {
        console.error("Erro ao carregar dados na página IA-Integrada (cliente-side):", err);
        setError(`Erro ao carregar dados: ${err.message || 'Erro desconhecido.'}`);
        setOllamaModels([]); // Limpa se houver erro
        setDocuments([]);     // Limpa se houver erro
        initializeDropdowns([], []); // Zera a seleção
      } finally {
        setIsLoadingPage(false);
      }
    };

    // Lógica para carregar dados: primeiro tenta das props, senão, faz fetch cliente-side
    if (data?.ollama_models && data?.documents) {
      setOllamaModels(data.ollama_models);
      setDocuments(data.documents);
      initializeDropdowns(data.ollama_models, data.documents);
      setIsLoadingPage(false);
    } else {
      fetchDataClient();
    }
  }, [data]); // Depende de 'data' para saber se foi pré-renderizado

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLlmResponse('');
    setError(null);
    setIsLoadingResponse(true);

    // Validação robusta ANTES de enviar
    if (!selectedModel || !selectedDocumentName || !question.trim()) { // Adicionado .trim() para pergunta vazia
      setError('Por favor, selecione um modelo, um documento e insira uma pergunta.');
      setIsLoadingResponse(false);
      return;
    }

    const payload = {
      file_name: selectedDocumentName,
      question: question,
      model: selectedModel,
    };

    try {
      const response = await fetch('/api/ollama', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Erro LLM: ${response.status} - ${errorText || 'Erro desconhecido.'}`);
      }

      const responseData = await response.json();
      if (responseData.error) {
        setError(responseData.error);
      } else {
        setLlmResponse(responseData.llm_response);
      }
    } catch (err: any) {
      console.error('Erro na requisição ao Ollama:', err);
      setError(`Erro ao obter resposta do LLM: ${err.message || 'Erro desconhecido.'}`);
    } finally {
      setIsLoadingResponse(false);
    }
  };

  if (isLoadingPage) {
    return <div className="loading-container"><h1>Carregando página de IA Integrada...</h1></div>;
  }

  // Exibir erro se não houver modelos OU documentos disponíveis
  const showNoOptionsError = ollamaModels.length === 0 || documents.length === 0;

  return (
    <div className="new-client-page-container">
      <h1 className="page-title">IA Integrada (Ollama)</h1>
      <p className="page-description">Faça perguntas sobre seus documentos usando modelos de linguagem locais.</p>

      {showNoOptionsError && (
        <p className="error-message" style={{ margin: '1rem 0' }}>
          Não foi possível carregar modelos Ollama ou documentos. Verifique se o Ollama está rodando e se há documentos no banco de dados.
        </p>
      )}

      <form onSubmit={handleSubmit} className="client-form">
        <div className="form-group">
          <label htmlFor="model-select" className="form-label">Modelo Ollama:</label>
          <select
            id="model-select"
            className="form-input"
            value={selectedModel}
            onChange={(e) => setSelectedModel(e.target.value)}
            required
            disabled={ollamaModels.length === 0} // Desabilita se não houver modelos
          >
            <option value="">{ollamaModels.length === 0 ? 'Carregando modelos...' : 'Selecione um modelo'}</option>
            {ollamaModels.map(model => (
              <option key={model.id} value={model.id}>{model.nome}</option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label htmlFor="document-select" className="form-label">Documento:</label>
          <select
            id="document-select"
            className="form-input"
            value={selectedDocumentName}
            onChange={(e) => setSelectedDocumentName(e.target.value)}
            required
            disabled={documents.length === 0} // Desabilita se não houver documentos
          >
            <option value="">{documents.length === 0 ? 'Carregando documentos...' : 'Selecione um documento'}</option>
            {documents.map(doc => (
              <option key={doc.id_documento} value={doc.nome_arquivo}>{doc.nome_arquivo}</option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label htmlFor="question-input" className="form-label">Sua Pergunta:</label>
          <textarea
            id="question-input"
            className="form-input"
            value={question}
            onChange={(e) => setQuestion(e.target.value)}
            rows={4}
            placeholder="Ex: Qual o valor total no contrato?"
            required
            disabled={showNoOptionsError} // Desabilita se não houver modelos/documentos
          />
        </div>

        <div className="form-actions" style={{ justifyContent: 'center' }}>
          <button type="submit" className="submit-button" disabled={isLoadingResponse || showNoOptionsError}>
            {isLoadingResponse ? 'Gerando resposta...' : 'Obter Resposta do LLM'}
          </button>
        </div>
      </form>

      {llmResponse && (
        <div className="response-section" style={{ marginTop: '2rem', padding: '1.5rem', backgroundColor: '#f0f8ff', borderRadius: '0.75rem', boxShadow: 'var(--shadow-light)' }}>
          <h2 style={{ fontSize: '1.5rem', color: 'var(--dark-text)', marginBottom: '1rem' }}>Resposta do LLM:</h2>
          <p style={{ whiteSpace: 'pre-wrap', color: '#333' }}>{llmResponse}</p>
        </div>
      )}

      {error && <p className="error-message" style={{ marginTop: '1rem' }}>{error}</p>}
    </div>
  );
}