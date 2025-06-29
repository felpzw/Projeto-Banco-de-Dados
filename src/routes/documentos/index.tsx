import { useState, useMemo, useEffect } from 'react';
import type { JSX } from 'react';
import { Link, useRouter } from 'tuono';
import type { TuonoRouteProps } from 'tuono';
import DocumentCard, { Document } from '../../components/DocumentCard';

interface DocumentsPageProps {
  documents: Document[];
}

export default function DocumentsPage({
  data,
  isLoading,
}: TuonoRouteProps<DocumentsPageProps>): JSX.Element {
  const router = useRouter();
  const [documents, setDocuments] = useState<Document[]>((data?.documents) ?? []);
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    const fetchDocuments = async () => {
      try {
        const response = await fetch('/api/documentos');
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const fetchedDocuments: Document[] = await response.json();
        setDocuments(fetchedDocuments);
      } catch (error) {
        console.error('Error fetching documents:', error);
      }
    };

    if (!data?.documents) {
      fetchDocuments();
    } else {
      setDocuments(data.documents);
    }
  }, [data]);

  const handleViewDetails = (documentId: string) => {
    router.push(`/documentos/${documentId}`);
  };

  const handleEditDocument = (documentId: string) => {
    router.push(`/documentos/edit/${documentId}`);
  };

  const handleDeleteDocument = async (documentId: string) => {
    const confirmDelete = confirm('Tem certeza que deseja excluir este documento?');
    if (!confirmDelete) {
      return;
    }

    try {
      const response = await fetch(`/api/documentos?id=${documentId}`, {
        method: 'DELETE',
      });

      if (response.ok) {
        setDocuments(prevDocuments => prevDocuments.filter(doc => doc.id_documento !== documentId));
      } else {
        const errorData = await response.json();
        alert(`Erro ao excluir documento: ${errorData.error || response.statusText}`);
      }
    } catch (error) {
      alert('Erro de rede ou servidor ao tentar excluir o documento.');
    }
  };

  if (isLoading) {
    return (
      <div className="loading-container">
        <h1>Carregando Documentos...</h1>
      </div>
    );
  }

  const filteredDocuments = useMemo(() => {
    if (!searchTerm) {
      return documents;
    }
    const lowerCaseSearchTerm = searchTerm.toLowerCase();
    return documents.filter(doc =>
      doc.nome_arquivo.toLowerCase().includes(lowerCaseSearchTerm) ||
      doc.descricao.toLowerCase().includes(lowerCaseSearchTerm) ||
      doc.id_caso.toLowerCase().includes(lowerCaseSearchTerm)
    );
  }, [documents, searchTerm]);

  return (
    <div className="clients-page-container">
      <div className="page-header">
        <h1 className="page-title">Gestão de Documentos</h1>
        <Link href="/documentos/new" className="add-button">
          + Adicionar Novo Documento
        </Link>
      </div>

      <div className="filters-section">
        <input
          type="text"
          placeholder="Buscar por nome, descrição ou ID do caso..."
          className="search-input"
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
      </div>

      <div className="client-list-grid">
        {filteredDocuments.length > 0 ? (
          filteredDocuments.map(document => (
            <DocumentCard
              key={document.id_documento}
              document={document}
              onViewDetails={handleViewDetails}
              onEdit={handleEditDocument}
              onDelete={handleDeleteDocument}
            />
          ))
        ) : (
          <p className="no-results-message">Nenhum documento encontrado com os critérios de busca.</p>
        )}
      </div>
    </div>
  );
}