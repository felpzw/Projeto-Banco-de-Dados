import { useRouter, Link } from 'tuono';
import { useEffect, useState } from 'react';
import type { JSX } from 'react';
import type { Case } from '../../components/CaseCard';

export default function CaseDetailsPage(): JSX.Element {
  const router = useRouter();
  const [caseItem, setCaseItem] = useState<Case | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  const id = router.pathname.split('/').pop();

  useEffect(() => {
    if (!id) {
      setIsLoading(false);
      return;
    }

    setIsLoading(true);
    fetch(`/api/casos?id=${id}`, {
      method: 'GET',
    })
      .then(async (res) => {
        const data = await res.json();
        if (data.error) {
          setError(data.error);
        } else {
          // O backend agora retorna o objeto diretamente para um ID específico
          setCaseItem(data);
        }
      })
      .catch((err) => {
        setError(`Erro ao buscar detalhes do caso: ${err.message}`);
      })
      .finally(() => {
        setIsLoading(false);
      });
  }, [id]);

  if (isLoading) {
    return <div className="loading-container"><h1>Carregando detalhes do caso...</h1></div>;
  }

  if (error) return <p className="error-message" style={{ margin: '2.5rem' }}>{error}</p>;
  if (!caseItem) return <p className="no-results-message" style={{ margin: '2.5rem' }}>Caso não encontrado.</p>;

  return (
    <div className="new-client-page-container"> {/* Reutilizando estilos de container */}
      <h1 className="page-title">Detalhes do Caso Jurídico</h1>
      <p className="page-description">Informações completas sobre o caso.</p>

      <div className="client-form" style={{ gap: '1rem' }}>
        <div className="form-group">
          <label className="form-label">ID Caso:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{caseItem.id_caso}</p>
        </div>
        <div className="form-group">
          <label className="form-label">Número Processo:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{caseItem.numero_processo || 'N/A'}</p>
        </div>
        <div className="form-group">
          <label className="form-label">Cliente:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{caseItem.cliente_nome} ({caseItem.cliente_email || 'N/A'})</p>
        </div>
        <div className="form-group">
          <label className="form-label">Advogado:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{caseItem.advogado_nome} (OAB: {caseItem.advogado_oab})</p>
        </div>
        <div className="form-group">
          <label className="form-label">Status:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{caseItem.status_descricao}</p>
        </div>
        {caseItem.nome_vara && (
          <div className="form-group">
            <label className="form-label">Vara Judicial:</label>
            <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{caseItem.nome_vara}</p>
          </div>
        )}
        {caseItem.categoria_descricao && (
          <div className="form-group">
            <label className="form-label">Categoria:</label>
            <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{caseItem.categoria_descricao}</p>
          </div>
        )}
        <div className="form-group">
          <label className="form-label">Descrição do Caso:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{caseItem.descricao || 'N/A'}</p>
        </div>
        <div className="form-group">
          <label className="form-label">Data de Abertura:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{new Date(caseItem.data_abertura).toLocaleDateString('pt-BR')}</p>
        </div>
        {caseItem.data_fechamento && (
          <div className="form-group">
            <label className="form-label">Data de Fechamento:</label>
            <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{new Date(caseItem.data_fechamento).toLocaleDateString('pt-BR')}</p>
          </div>
        )}
      </div>

      <div className="form-actions" style={{ justifyContent: 'flex-start' }}>
        <Link href={`/casos/edit/${caseItem.id_caso}`} className="submit-button" style={{ backgroundColor: '#ffc107', color: '#333' }}>
          Editar Caso
        </Link>
        <Link href="/casos" className="cancel-button">
          Voltar para Casos
        </Link>
      </div>
    </div>
  );
}