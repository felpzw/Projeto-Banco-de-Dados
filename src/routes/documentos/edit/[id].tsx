import { useState, useEffect } from 'react';
import type { JSX } from 'react';
import { Link, useRouter } from 'tuono';
import type { Document } from '../../../components/DocumentCard'; // Reutiliza a interface

export default function EditDocumentPage(): JSX.Element {
  const router = useRouter();
  const id = router.pathname.split('/').pop();

  const [formData, setFormData] = useState<Document>({
    id_documento: '',
    id_caso: '',
    descricao: '',
    data_envio: '',
    tipo: '',
    nome_arquivo: '',
  });
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    if (!id) {
      setIsLoading(false);
      return;
    }

    const fetchDocumentData = async () => {
      setIsLoading(true);
      try {
        const response = await fetch(`/api/documentos?id=${id}`);
        const data: Document & { error?: string } = await response.json();

        if (data.error) {
          setError(data.error);
        } else {
          setFormData({
            id_documento: data.id_documento,
            id_caso: data.id_caso.toString(), // Converter para string para o input type="number"
            descricao: data.descricao,
            data_envio: data.data_envio ? new Date(data.data_envio).toISOString().split('T')[0] : '', // Formato YYYY-MM-DD
            tipo: data.tipo,
            nome_arquivo: data.nome_arquivo,
          });
        }
      } catch (err) {
        console.error('Erro ao carregar dados do documento para edição:', err);
        setError('Erro ao carregar dados do documento para edição.');
      } finally {
        setIsLoading(false);
      }
    };

    fetchDocumentData();
  }, [id]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
    const { name, value } = e.target;
    setFormData(prev => ({ ...prev, [name]: value }));
  };

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setMessage('');
    setError('');

    // Basic validation
    if (!formData.id_caso || !formData.descricao || !formData.data_envio || !formData.tipo || !formData.nome_arquivo) {
      setError('Por favor, preencha todos os campos obrigatórios.');
      return;
    }

    const queryParams = new URLSearchParams({
      id: id!, // ID do documento a ser atualizado
      id_caso: formData.id_caso.toString(),
      descricao: formData.descricao,
      data_envio: formData.data_envio,
      tipo: formData.tipo,
      nome_arquivo: formData.nome_arquivo,
    });

    try {
      const response = await fetch(`/api/documentos?${queryParams.toString()}`, {
        method: 'PUT',
      });

      if (response.ok) {
        setMessage('Documento atualizado com sucesso!');
        setTimeout(() => {
          router.push(`/documentos/${id}`); // Redireciona para a página de detalhes do documento
        }, 1500);
      } else {
        const errorText = await response.text();
        console.error('Erro ao atualizar documento:', response.status, errorText);
        setError(`Erro ao atualizar documento: ${response.status} - ${errorText || 'Erro desconhecido.'}`);
      }
    } catch (err) {
      console.error('Erro de rede ou servidor ao atualizar documento:', err);
      setError('Erro de rede ou servidor ao tentar atualizar o documento.');
    }
  };

  if (isLoading) {
    return <div className="loading-container"><h1>Carregando dados para edição...</h1></div>;
  }

  if (error && !message) {
    return <p className="error-message" style={{ margin: '2.5rem' }}>{error}</p>;
  }

  return (
    <div className="new-client-page-container">
      <h1 className="page-title">Editar Documento</h1>
      <p className="page-description">Altere os dados do documento e salve as modificações.</p>

      <form onSubmit={handleSubmit} className="client-form">
        <div className="form-group">
          <label htmlFor="id_caso" className="form-label">ID do Caso:</label>
          <input
            type="number"
            id="id_caso"
            name="id_caso"
            className="form-input"
            value={formData.id_caso}
            onChange={handleChange}
            required
          />
        </div>
        <div className="form-group">
          <label htmlFor="nome_arquivo" className="form-label">Nome do Arquivo:</label>
          <input
            type="text"
            id="nome_arquivo"
            name="nome_arquivo"
            className="form-input"
            value={formData.nome_arquivo}
            onChange={handleChange}
            required
          />
        </div>
        <div className="form-group">
          <label htmlFor="descricao" className="form-label">Descrição:</label>
          <textarea
            id="descricao"
            name="descricao"
            className="form-input"
            value={formData.descricao}
            onChange={handleChange}
            required
            rows={4}
          />
        </div>
        <div className="form-group">
          <label htmlFor="tipo" className="form-label">Tipo (Ex: PDF, JPG, DOCX):</label>
          <input
            type="text"
            id="tipo"
            name="tipo"
            className="form-input"
            value={formData.tipo}
            onChange={handleChange}
            required
          />
        </div>
        <div className="form-group">
          <label htmlFor="data_envio" className="form-label">Data de Envio:</label>
          <input
            type="date"
            id="data_envio"
            name="data_envio"
            className="form-input"
            value={formData.data_envio}
            onChange={handleChange}
            required
          />
        </div>

        {message && <p className="success-message">{message}</p>}
        {error && <p className="error-message">{error}</p>}

        <div className="form-actions">
          <button type="submit" className="submit-button">
            Salvar Alterações
          </button>
          <Link href={`/documentos/${id}`} className="cancel-button">
            Cancelar
          </Link>
        </div>
      </form>
    </div>
  );
}