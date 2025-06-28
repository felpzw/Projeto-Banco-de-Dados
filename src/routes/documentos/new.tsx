import { useState } from 'react';
import type { JSX } from 'react';
import { Link, useRouter } from 'tuono';

export default function NewDocumentPage(): JSX.Element {
  const router = useRouter();
  const [formData, setFormData] = useState({
    id_caso: '',
    descricao: '',
    data_envio: '', // Formato YYYY-MM-DD
    tipo: '',
    nome_arquivo: '',
  });
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

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

    const queryParams = new URLSearchParams(formData);

    try {
      const response = await fetch(`/api/documentos?${queryParams.toString()}`, {
        method: 'POST',
      });

      if (response.ok) {
        setMessage('Documento adicionado com sucesso!');
        setFormData({ // Clear form
          id_caso: '',
          descricao: '',
          data_envio: '',
          tipo: '',
          nome_arquivo: '',
        });
        setTimeout(() => {
          router.push('/documentos');
        }, 1500);
      } else {
        const errorText = await response.text();
        console.error('Erro ao adicionar documento:', response.status, errorText);
        setError(`Erro ao adicionar documento: ${response.status} - ${errorText || 'Erro desconhecido.'}`);
      }
    } catch (err) {
      console.error('Erro de rede ou servidor ao adicionar documento:', err);
      setError('Erro de rede ou servidor ao tentar adicionar o documento.');
    }
  };

  return (
    <div className="new-client-page-container"> {/* Reutilizando container e estilos de form */}
      <h1 className="page-title">Adicionar Novo Documento</h1>
      <p className="page-description">Preencha os dados abaixo para cadastrar um novo documento.</p>

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
            rows={4} // Aumentar o tamanho do campo de texto
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
            Adicionar Documento
          </button>
          <Link href="/documentos" className="cancel-button">
            Cancelar
          </Link>
        </div>
      </form>
    </div>
  );
}