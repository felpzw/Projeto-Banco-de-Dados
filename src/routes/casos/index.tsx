import { useState, useMemo, useEffect } from 'react';
import type { JSX } from 'react';
import { Link, useRouter } from 'tuono';
import type { TuonoRouteProps } from 'tuono';
import CaseCard, { Case } from '../../components/CaseCard';

interface CasesPageProps {
  cases: Case[];
}

export default function CasesPage({
  data,
  isLoading: propIsLoading,
}: TuonoRouteProps<CasesPageProps>): JSX.Element {
  const router = useRouter();
  const [cases, setCases] = useState<Case[]>(data?.cases || []);
  const [searchTerm, setSearchTerm] = useState('');
  const [isLoading, setIsLoading] = useState(propIsLoading);

  useEffect(() => {
    const fetchCases = async () => {
      setIsLoading(true);
      try {
        const response = await fetch('/api/casos');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const fetchedCases: Case[] = await response.json();
        setCases(fetchedCases);
      } catch (error) {
        console.error('Error fetching cases:', error);
      } finally {
        setIsLoading(false);
      }
    };

    if (!data?.cases) {
      fetchCases();
    } else {
      setCases(data.cases);
      setIsLoading(false);
    }
  }, [data]);

  const handleViewDetails = (caseId: string) => {
    router.push(`/casos/${caseId}`);
  };

  const handleEditCase = (caseId: string) => {
    router.push(`/casos/edit/${caseId}`);
  };

  const handleDeleteCase = async (caseId: string) => {
    const confirmDelete = confirm('Tem certeza que deseja excluir este caso? Todos os dados relacionados (andamentos, audiências, peças, documentos, tarefas) serão APAGADOS!');
    if (!confirmDelete) {
      return;
    }

    try {
      const response = await fetch(`/api/casos?id=${caseId}`, {
        method: 'DELETE',
      });

      if (response.ok) {
        setCases(prevCases => prevCases.filter(c => c.id_caso !== caseId));
      } else {
        const errorData = await response.json();
        alert(`Erro ao excluir caso: ${errorData.error || response.statusText}`);
      }
    } catch (error) {
      alert('Erro de rede ou servidor ao tentar excluir o caso.');
    }
  };

  const filteredCases = useMemo(() => {
    if (!searchTerm) {
      return cases;
    }
    const lowerCaseSearchTerm = searchTerm.toLowerCase();
    return cases.filter(caseItem =>
      (caseItem.numero_processo?.toLowerCase() || '').includes(lowerCaseSearchTerm) ||
      (caseItem.cliente_nome?.toLowerCase() || '').includes(lowerCaseSearchTerm) ||
      (caseItem.advogado_nome?.toLowerCase() || '').includes(lowerCaseSearchTerm) ||
      (caseItem.status_descricao?.toLowerCase() || '').includes(lowerCaseSearchTerm) ||
      (caseItem.descricao?.toLowerCase() || '').includes(lowerCaseSearchTerm)
    );
  }, [cases, searchTerm]);

  return (
    <div className="clients-page-container">
      <div className="page-header">
        <h1 className="page-title">Gestão de Casos Jurídicos</h1>
        <Link href="/casos/new" className="add-button">
          + Adicionar Novo Caso
        </Link>
      </div>

      <div className="filters-section">
        <input
          type="text"
          placeholder="Buscar por nº processo, cliente, advogado, status ou descrição..."
          className="search-input"
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
      </div>

      <div className="client-list-grid">
        {filteredCases.length > 0 ? (
          filteredCases.map(caseItem => (
            <CaseCard
              key={caseItem.id_caso}
              caseItem={caseItem}
              onViewDetails={handleViewDetails}
              onEdit={handleEditCase}
              onDelete={handleDeleteCase}
            />
          ))
        ) : (
          <p className="no-results-message">Nenhum caso jurídico encontrado com os critérios de busca.</p>
        )}
      </div>
    </div>
  );
}