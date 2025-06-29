import type { JSX } from 'react';

// Interface para os dados do caso com JOINS
export interface Case {
  id_caso: string;
  descricao?: string;
  numero_processo?: string;
  data_abertura: string;
  data_fechamento?: string;

  // Cliente
  id_cliente: string;
  cliente_nome: string;
  cliente_email?: string;

  // Advogado
  id_advogado: string;
  advogado_nome: string;
  advogado_oab: string;

  // Status
  id_status: string;
  status_descricao: string;

  // Vara Judicial (Opcional)
  id_vara_judicial?: string;
  nome_vara?: string;

  // Categoria Caso (Opcional)
  id_categoria_caso?: string;
  categoria_descricao?: string;
}

interface CaseCardProps {
  caseItem: Case; // Renomeado de 'case' para 'caseItem' para evitar conflito com palavra reservada
  onViewDetails?: (caseId: string) => void;
  onEdit?: (caseId: string) => void;
  onDelete?: (caseId: string) => Promise<void>;
}

export default function CaseCard({ caseItem, onViewDetails, onEdit, onDelete }: CaseCardProps): JSX.Element {
  const handleViewClick = () => {
    if (onViewDetails) {
      onViewDetails(caseItem.id_caso);
    }
  };

  const handleEditClick = () => {
    if (onEdit) {
      onEdit(caseItem.id_caso);
    }
  };

  const handleDeleteClick = () => {
    if (onDelete) {
      onDelete(caseItem.id_caso);
    }
  };

  return (
    <div className="client-card"> {/* Reutilizando estilos de card existente */}
      <h3 className="client-card-name">Processo: {caseItem.numero_processo || 'N/A'}</h3>
      <p className="client-card-detail">
        <strong>Cliente:</strong> {caseItem.cliente_nome}
      </p>
      <p className="client-card-detail">
        <strong>Advogado:</strong> {caseItem.advogado_nome} ({caseItem.advogado_oab})
      </p>
      <p className="client-card-detail">
        <strong>Status:</strong> {caseItem.status_descricao}
      </p>
      <p className="client-card-detail">
        <strong>Descrição:</strong> {caseItem.descricao || 'N/A'}
      </p>
      <p className="client-card-detail">
        <strong>Abertura:</strong> {new Date(caseItem.data_abertura).toLocaleDateString('pt-BR')}
      </p>
      {caseItem.data_fechamento && (
        <p className="client-card-detail">
          <strong>Fechamento:</strong> {new Date(caseItem.data_fechamento).toLocaleDateString('pt-BR')}
        </p>
      )}
      {caseItem.nome_vara && (
        <p className="client-card-detail">
          <strong>Vara:</strong> {caseItem.nome_vara}
        </p>
      )}
      {caseItem.categoria_descricao && (
        <p className="client-card-detail">
          <strong>Categoria:</strong> {caseItem.categoria_descricao}
        </p>
      )}

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