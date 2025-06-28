import { useState, useMemo, useEffect } from 'react';
import type { JSX } from 'react';
import { Link, useRouter } from 'tuono'; // Import useRouter
import type { TuonoRouteProps } from 'tuono';
import ClientCard, { Client } from '../../components/ClientCard'; // Importa o componente e a interface

// Definindo a interface para os dados recebidos do handler (simulado ou do backend)
interface ClientsPageProps {
  clientes: Cliente[];
}

// Updated Client interface to reflect optional cpf/cnpj and mandatory email/endereco
interface Cliente {
  id_cliente: string;
  nome: string;
  email: string;
  telefone: string;
  endereco: string;
  data_cadastro: string; // ISO 8601 string
  cpf?: string; // Optional for Pessoa_Fisica
  cnpj?: string; // Optional for Pessoa_Juridica
}

export default function ClientsPage({
  data,
  isLoading,
}: TuonoRouteProps<ClientsPageProps>): JSX.Element {
  const router = useRouter();
  const [clients, setClients] = useState<Cliente[]>((data?.clientes) ?? []);
  const [searchTerm, setSearchTerm] = useState('');

  const handleViewDetails = (clientId: string) => {
    console.log(`Ver detalhes do cliente com ID: ${clientId}`); // Debug log
    router.push(`/clientes/${clientId}`); // Navigate to client details page (hypothetical)
  };

  const handleEditClient = (clientId: string) => {
    console.log(`Editando cliente com ID: ${clientId}`); // Debug log
    router.push(`/clientes/edit/${clientId}`); // Navigate to edit page
  };

  const handleDeleteClient = async (clientId: string) => {
    console.log(`Tentando excluir cliente com ID: ${clientId}`); // Debug log

    const confirmDelete = confirm('Tem certeza que deseja excluir este cliente?');
    if (!confirmDelete) {
      console.log('Exclusão cancelada.'); // Debug log
      return;
    }

    try {
      // Perform the fetch request to delete the client
      const response = await fetch(`/api/clientes?id=${clientId}`, {
        method: 'DELETE',
      });

      if (response.ok) {
        console.log(`Cliente com ID ${clientId} excluído com sucesso!`); // Debug log
        // Update the state to remove the deleted client
        setClients(prevClients => prevClients.filter(client => client.id_cliente !== clientId));
      } else {
        const errorData = await response.json();
        console.error(`Falha ao excluir cliente ${clientId}:`, errorData.error || response.statusText); // Debug log
        alert(`Erro ao excluir cliente: ${errorData.error || 'Erro desconhecido'}`);
      }
    } catch (error) {
      console.error(`Erro na requisição de exclusão para o cliente ${clientId}:`, error); // Debug log
      alert('Erro de rede ou servidor ao tentar excluir o cliente.');
    }
  };

  if (isLoading) {
    return (
      <div className="loading-container">
        <h1>Carregando Clientes...</h1>
      </div>
    );
  }

  // Filter clients based on search term (name, email, phone, cpf, cnpj)
  const filteredClients = useMemo(() => {
    if (!searchTerm) {
      return clients;
    }
    const lowerCaseSearchTerm = searchTerm.toLowerCase();
    return clients.filter(client =>
      client.nome.toLowerCase().includes(lowerCaseSearchTerm) ||
      client.email.toLowerCase().includes(lowerCaseSearchTerm) ||
      client.telefone.includes(lowerCaseSearchTerm) ||
      (client.cpf && client.cpf.includes(lowerCaseSearchTerm)) || // Check cpf if exists
      (client.cnpj && client.cnpj.includes(lowerCaseSearchTerm))   // Check cnpj if exists
    );
  }, [clients, searchTerm]);

  return (
    <div className="clients-page-container">
      <div className="page-header">
        <h1 className="page-title">Gestão de Clientes</h1>
        <Link href="/clientes/new" className="add-button">
          + Adicionar Novo Cliente
        </Link>
      </div>

      <div className="filters-section">
        <input
          type="text"
          placeholder="Buscar por nome, email, telefone, CPF ou CNPJ..."
          className="search-input"
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          onKeyUp={() => console.log('Searching for:', searchTerm)}
        />
      </div>

      <div className="client-list-grid">
        {filteredClients.length > 0 ? (
          filteredClients.map(client => (
            <ClientCard
              key={client.id_cliente}
              client={client}
              onViewDetails={handleViewDetails}
              onEdit={handleEditClient}
              onDelete={handleDeleteClient}
            />
          ))
        ) : (
          <p className="no-results-message">Nenhum cliente encontrado com os critérios de busca.</p>
        )}
      </div>
    </div>
  );
}
