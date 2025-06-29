import type { JSX } from 'react';

export interface Document {
  id_documento: string;
  id_caso: string; // ID do caso ao qual o documento pertence
  descricao: string;
  data_envio: string; // ISO 8601 string
  nome_arquivo: string; // Nome original do arquivo (com extensão)
}

interface DocumentCardProps {
  document: Document;
  onViewDetails?: (documentId: string) => void;
  onEdit?: (documentId: string) => void;
  onDelete?: (documentId: string) => Promise<void>;
}

export default function DocumentCard({ document, onViewDetails, onEdit, onDelete }: DocumentCardProps): JSX.Element {
  const handleViewClick = () => {
    if (onViewDetails) {
      onViewDetails(document.id_documento);
    }
  };

  const handleEditClick = () => {
    if (onEdit) {
      onEdit(document.id_documento);
    }
  };

  const handleDeleteClick = () => {
    if (onDelete) {
      onDelete(document.id_documento);
    }
  };

  return (
    <div className="client-card">
      <h3 className="client-card-name">Documento: {document.nome_arquivo}</h3>
      <p className="client-card-detail">
        <strong>ID Doc:</strong> {document.id_documento}
      </p>
      <p className="client-card-detail">
        <strong>ID Caso:</strong> {document.id_caso}
      </p>
      <p className="client-card-detail">
        <strong>Descrição:</strong> {document.descricao}
      </p>
      <p className="client-card-detail">
        <strong>Data Envio:</strong> {document.data_envio ? new Date(document.data_envio).toLocaleDateString('pt-BR') : 'Não informado'}
      </p>
      <div className="client-card-actions">
        {onViewDetails && (
          <button className="client-card-button" onClick={handleViewClick}>
            Ver Detalhes
          </button>
        )}
        {onEdit && (
          <button className="client-card-button edit-button" onClick={handleEditClick}>
            Editar
          </button>
        )}
        {onDelete && (
          <button className="client-card-button delete-button" onClick={handleDeleteClick}>
            Excluir
          </button>
        )}
      </div>
    </div>
  );
}