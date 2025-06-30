import { useState } from 'react';
import type { JSX } from 'react';
import { Link } from 'tuono';

export default function ConfiguracoesPage(): JSX.Element {
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<string | null>(null); // Para indicar qual botão está carregando

  const handleDbOperation = async (method: string, endpoint: string, buttonLabel: string) => {
    setMessage(null);
    setError(null);
    setIsLoading(buttonLabel); // Define qual botão está carregando

    try {
      const response = await fetch(endpoint, {
        method: method,
      });

      if (response.ok) {
        const data = await response.json();
        setMessage(`Sucesso na operação "${buttonLabel}": ${data.message || 'Operação concluída.'}`);
      } else {
        const errorText = await response.text();
        let errorMessage = `Falha na operação "${buttonLabel}": Status ${response.status}`;
        try {
          const errorJson = JSON.parse(errorText);
          errorMessage += ` - ${errorJson.error || 'Erro desconhecido.'}`;
        } catch {
          errorMessage += ` - ${errorText || 'Erro desconhecido.'}`;
        }
        setError(errorMessage);
        console.error(`Erro na operação "${buttonLabel}":`, errorText);
      }
    } catch (err: any) {
      setError(`Erro de rede/servidor na operação "${buttonLabel}": ${err.message || 'Erro desconhecido.'}`);
      console.error(`Erro de rede/servidor na operação "${buttonLabel}":`, err);
    } finally {
      setIsLoading(null); // Limpa o estado de carregamento
    }
  };

  return (
    <div className="new-client-page-container"> {/* Reutiliza o estilo do container */}
      <h1 className="page-title">Configurações de Banco de Dados</h1>
      <p className="page-description">
        Utilize estes botões para gerenciar o estado do seu banco de dados durante o desenvolvimento.
        **Cuidado**: A operação DELETE limpa todos os dados!
      </p>

      <div className="form-actions" style={{ flexDirection: 'column', gap: '1.5rem', alignItems: 'center' }}>
        {/* Botão DELETE */}
        <button
          className="submit-button delete-button"
          onClick={() => handleDbOperation('DELETE', '/api/clean', 'LIMPAR DB (DEBUG)')}
          disabled={isLoading === 'LIMPAR DB (DEBUG)'}
        >
          {isLoading === 'LIMPAR DB (DEBUG)' ? 'Limpando...' : 'LIMPAR DB (DEBUG)'}
        </button>

        {/* Botão POST */}
        <button
          className="submit-button"
          onClick={() => handleDbOperation('POST', '/api/init', 'INICIAR DB (DEBUG)')}
          disabled={isLoading === 'INICIAR DB (DEBUG)'}
        >
          {isLoading === 'INICIAR DB (DEBUG)' ? 'Iniciando...' : 'INICIAR DB (DEBUG)'}
        </button>

        {/* Botão PUT */}
        <button
          className="submit-button edit-button" // Usando estilo 'edit-button' para cor amarela
          onClick={() => handleDbOperation('PUT', '/api/populate_db', 'POPULAR DB (DEBUG)')}
          disabled={isLoading === 'POPULAR DB (DEBUG)'}
        >
          {isLoading === 'POPULAR DB (DEBUG)' ? 'Populando...' : 'POPULAR DB (DEBUG)'}
        </button>
      </div>

      {message && <p className="success-message" style={{ marginTop: '2rem' }}>{message}</p>}
      {error && <p className="error-message" style={{ marginTop: '2rem' }}>{error}</p>}
    </div>
  );
}