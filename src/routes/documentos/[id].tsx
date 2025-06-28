import { useRouter, Link } from 'tuono';
import { useEffect, useState } from 'react';
import type { JSX } from 'react';
import type { Document } from '../../components/DocumentCard'; // Reutiliza a interface

export default function DocumentoPage(): JSX.Element {
  const router = useRouter();
  const [documento, setDocumento] = useState<Document | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  const id = router.pathname.split('/').pop();

  useEffect(() => {
    if (!id) {
      setIsLoading(false);
      return;
    }

    setIsLoading(true);
    fetch(`/api/documentos?id=${id}`, {
      method: 'GET',
    })
      .then(async (res) => {
        const data = await res.json();
        if (data.error) {
          setError(data.error);
        } else {
          setDocumento(data);
        }
      })
      .catch((err) => {
        setError(`Erro ao buscar documento: ${err.message}`);
      })
      .finally(() => {
        setIsLoading(false);
      });
  }, [id]);

  if (isLoading) {
    return <div className="loading-container"><h1>Carregando detalhes do documento...</h1></div>;
  }

  if (error) return <p className="error-message" style={{ margin: '2.5rem' }}>{error}</p>;
  if (!documento) return <p className="no-results-message" style={{ margin: '2.5rem' }}>Documento não encontrado.</p>;

  return (
    <div className="new-client-page-container"> {/* Reutilizando container e estilos de form */}
      <h1 className="page-title">Detalhes do Documento</h1>
      <p className="page-description">Informações completas sobre o documento.</p>

      <div className="client-form" style={{ gap: '1rem' }}>
        <div className="form-group">
          <label className="form-label">ID Documento:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{documento.id_documento}</p>
        </div>
        <div className="form-group">
          <label className="form-label">ID Caso:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{documento.id_caso}</p>
        </div>
        <div className="form-group">
          <label className="form-label">Nome do Arquivo:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{documento.nome_arquivo}</p>
        </div>
        <div className="form-group">
          <label className="form-label">Descrição:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{documento.descricao}</p>
        </div>
        <div className="form-group">
          <label className="form-label">Tipo:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{documento.tipo}</p>
        </div>
        <div className="form-group">
          <label className="form-label">Data de Envio:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>
            {documento.data_envio ? new Date(documento.data_envio).toLocaleDateString('pt-BR') : 'Não informado'}
          </p>
        </div>
      </div>

      <div className="form-actions" style={{ justifyContent: 'flex-start' }}>
        <Link href={`/documentos/edit/${documento.id_documento}`} className="submit-button" style={{ backgroundColor: '#ffc107', color: '#333' }}>
          Editar Documento
        </Link>
        <Link href="/documentos" className="cancel-button">
          Voltar para Documentos
        </Link>
      </div>
    </div>
  );
}