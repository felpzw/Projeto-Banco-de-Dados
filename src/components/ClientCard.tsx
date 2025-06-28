import type { JSX } from 'react'

// Definindo a interface para as propriedades do cliente
export interface Client {
  id_cliente: string
  nome: string // Nome do cliente
  email: string // Email do cliente
  endereco: string
  telefone: string
  data_cadastro: string // Formato string ISO 8601
  cpf?: string; // Opcional para Pessoa Física
  cnpj?: string; // Opcional para Pessoa Jurídica
}

interface ClientCardProps {
  client: Client
  onViewDetails?: (clientId: string) => void // Opcional, para ver detalhes
  onEdit?: (clientId: string) => void // Para edição
  onDelete?: (clientId: string) => Promise<void> // Para exclusão
}

export default function ClientCard({ client, onViewDetails, onEdit, onDelete }: ClientCardProps): JSX.Element {
  const handleViewClick = () => {
    if (onViewDetails) {
      onViewDetails(client.id_cliente)
    }
  }

  const handleEditClick = () => {
    if (onEdit) {
      onEdit(client.id_cliente)
    }
  }

  const handleDeleteClick = () => {
    if (onDelete) {
      onDelete(client.id_cliente)
    }
  }

  return (
    <div className="client-card">
      <h3 className="client-card-name">{client.nome}</h3>
      <p className="client-card-detail">
        <strong>ID:</strong> {client.id_cliente}
      </p>
      <p className="client-card-detail">
        <strong>Email:</strong> {client.email}
      </p>
      {client.cpf && ( // Conditionally display CPF
        <p className="client-card-detail">
          <strong>CPF:</strong> {client.cpf}
        </p>
      )}
      {client.cnpj && ( // Conditionally display CNPJ
        <p className="client-card-detail">
          <strong>CNPJ:</strong> {client.cnpj}
        </p>
      )}
      <p className="client-card-detail">
        <strong>Endereço:</strong> {client.endereco}
      </p>
      <p className="client-card-detail">
        <strong>Telefone:</strong> {client.telefone}
      </p>
      <p className="client-card-detail">
        <strong>Cadastro:</strong> {new Date(client.data_cadastro).toLocaleDateString('pt-BR')}
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
  )
}
