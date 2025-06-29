import { useState, useEffect, useCallback } from 'react';
import type { JSX } from 'react';
import { Link, useRouter } from 'tuono';
import type { Document } from '../../../components/DocumentCard';

export default function EditDocumentPage(): JSX.Element {
  const router = useRouter();
  const id = router.pathname.split('/').pop();

  const [formData, setFormData] = useState<Document>({
    id_documento: '',
    id_caso: '',
    descricao: '',
    data_envio: '',
    nome_arquivo: '',
  });
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [isDragOver, setIsDragOver] = useState(false);

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
            id_caso: data.id_caso.toString(),
            descricao: data.descricao,
            data_envio: data.data_envio ? new Date(data.data_envio).toISOString().split('T')[0] : '',
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

  // Correção: Tipagem explícita para 'prev' na função handleChange
  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
    const { name, value } = e.target;
    setFormData((prev: Document) => ({ ...prev, [name]: value })); // Adicionado ': Document' a 'prev'
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      const file = e.target.files[0];
      setSelectedFile(file);
      setFormData(prev => ({
        ...prev,
        nome_arquivo: file.name,
      }));
      setError('');
    } else {
      setSelectedFile(null);
      setFormData(prev => ({
        ...prev,
        nome_arquivo: formData.nome_arquivo, // Mantém o nome_arquivo existente se nenhum novo for selecionado
      }));
    }
  };

  const handleDragOver = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback(() => {
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setIsDragOver(false);

    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      const file = e.dataTransfer.files[0];
      setSelectedFile(file);
      setFormData(prev => ({
        ...prev,
        nome_arquivo: file.name,
      }));
      setError('');
    }
  }, []);

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setMessage('');
    setError('');

    if (!formData.id_caso || !formData.descricao || !formData.data_envio || !formData.nome_arquivo) {
      setError('Por favor, preencha todos os campos obrigatórios (incluindo nome do arquivo).');
      return;
    }

    let base64String: string | null = null;
    if (selectedFile) {
      const reader = new FileReader();
      reader.readAsArrayBuffer(selectedFile);

      await new Promise<void>((resolve, reject) => {
        reader.onloadend = () => {
          if (reader.result) {
            base64String = btoa(
              new Uint8Array(reader.result as ArrayBuffer)
                .reduce((data, byte) => data + String.fromCharCode(byte), '')
            );
            resolve();
          } else {
            reject(new Error('Erro ao ler o arquivo selecionado.'));
          }
        };
        reader.onerror = () => {
          reject(new Error('Erro ao ler o arquivo.'));
        };
      }).catch(err => {
        setError(err.message);
        return;
      });

      if (error) return;
    }

    const payload = {
      id: parseInt(formData.id_documento),
      id_caso: parseInt(formData.id_caso),
      descricao: formData.descricao,
      data_envio: formData.data_envio,
      nome_arquivo: formData.nome_arquivo,
      arquivo_base64: base64String, // Será null se nenhum novo arquivo for selecionado
    };

    try {
      const response = await fetch(`/api/documentos`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
      });

      if (response.ok) {
        setMessage('Documento atualizado com sucesso!');
        setTimeout(() => {
          router.push(`/documentos/${id}`);
        }, 1500);
      } else {
        const errorText = await response.text();
        console.error('Erro ao atualizar documento:', response.status, errorText);
        try {
          const errorJson = JSON.parse(errorText);
          setError(`Erro ao atualizar documento: ${response.status} - ${errorJson.error || 'Erro desconhecido.'}`);
        } catch {
          setError(`Erro ao atualizar documento: ${response.status} - ${errorText || 'Erro desconhecido.'}`);
        }
      }
    } catch (err) {
      console.error('Erro de rede ou servidor ao atualizar documento:', err);
      setError('Erro de rede ou servidor ao tentar atualizar o documento.');
    }
  };

  const handleDownload = async () => {
    if (!formData.id_documento) {
      setError('ID do documento não disponível para download.');
      return;
    }
    try {
      const response = await fetch(`/api/documentos?id=${formData.id_documento}&download=true`, {
        method: 'GET',
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Falha ao baixar documento: ${response.status} - ${errorText || 'Erro desconhecido'}`);
      }

      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = formData.nome_arquivo || 'documento';
      document.body.appendChild(a);
      a.click();
      a.remove();
      window.URL.revokeObjectURL(url);
    } catch (err: any) {
      console.error('Erro ao baixar documento:', err);
      setError(`Erro ao baixar documento: ${err.message}`);
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
      <p className="page-description">Altere os dados do documento e faça upload de um novo arquivo, se desejar.</p>

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

        {/* Área de Drag and Drop para re-upload */}
        <div
          className={`form-group drop-zone ${isDragOver ? 'drag-over' : ''}`}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          onClick={() => document.getElementById('file_input')?.click()}
          style={{
            border: `2px dashed ${isDragOver ? '#007bff' : '#ccc'}`,
            borderRadius: '0.5rem',
            padding: '2rem',
            textAlign: 'center',
            cursor: 'pointer',
            backgroundColor: isDragOver ? '#e0f2fe' : 'transparent',
            transition: 'background-color 0.2s ease, border-color 0.2s ease',
            color: '#666',
          }}
        >
          {selectedFile ? (
            <p>Novo arquivo selecionado: <strong>{selectedFile.name}</strong></p>
          ) : (
            <p>Arraste e solte um novo arquivo aqui para substituir, ou clique para selecionar. (Deixe vazio para manter o arquivo atual)</p>
          )}
          <input
            type="file"
            id="file_input"
            name="file_upload"
            onChange={handleFileChange}
            style={{ display: 'none' }}
          />
        </div>
        {!selectedFile && (
            <button
              type="button"
              className="client-card-button"
              onClick={handleDownload}
              style={{ backgroundColor: '#28a745', color: 'white', marginTop: '1rem' }}
            >
              Download Arquivo Atual
            </button>
          )}


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