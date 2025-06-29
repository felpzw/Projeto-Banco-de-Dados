import { useState, useCallback } from 'react';
import type { JSX } from 'react';
import { Link, useRouter } from 'tuono';

export default function NewDocumentPage(): JSX.Element {
  const router = useRouter();
  const [formData, setFormData] = useState({
    id_caso: '',
    descricao: '',
    data_envio: '', // Formato AAAA-MM-DD
    nome_arquivo: '', // Nome original do arquivo (com extensão)
  });
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [isDragOver, setIsDragOver] = useState(false);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
    const { name, value } = e.target;
    setFormData(prev => ({ ...prev, [name]: value }));
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
        nome_arquivo: '',
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

    if (!formData.id_caso || !formData.descricao || !formData.data_envio || !selectedFile) {
      setError('Por favor, preencha todos os campos obrigatórios e selecione um arquivo.');
      return;
    }

    // Leitura do arquivo como ArrayBuffer e conversão para Base64
    const reader = new FileReader();
    reader.readAsArrayBuffer(selectedFile);

    reader.onloadend = async () => {
      if (reader.result) {
        const base64String = btoa(
          new Uint8Array(reader.result as ArrayBuffer)
            .reduce((data, byte) => data + String.fromCharCode(byte), '')
        );

        // Objeto JSON a ser enviado no corpo da requisição
        const payload = {
          id_caso: parseInt(formData.id_caso), // Converter para número
          descricao: formData.descricao,
          data_envio: formData.data_envio,
          nome_arquivo: formData.nome_arquivo,
          arquivo_base64: base64String, // Envia o conteúdo Base64
        };

        try {
          const response = await fetch(`/api/documentos`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json', // Importante: indica que o corpo é JSON
            },
            body: JSON.stringify(payload), // Envia o JSON stringificado no corpo
          });

          if (response.ok) {
            setMessage('Documento adicionado com sucesso!');
            setFormData({
              id_caso: '',
              descricao: '',
              data_envio: '',
              nome_arquivo: '',
            });
            setSelectedFile(null);
            setTimeout(() => {
              router.push('/documentos');
            }, 1500);
          } else {
            const errorText = await response.text();
            console.error('Erro ao adicionar documento:', response.status, errorText);
            try {
              const errorJson = JSON.parse(errorText);
              setError(`Erro ao adicionar documento: ${response.status} - ${errorJson.error || 'Erro desconhecido.'}`);
            } catch {
              setError(`Erro ao adicionar documento: ${response.status} - ${errorText || 'Erro desconhecido.'}`);
            }
          }
        } catch (err) {
          console.error('Erro de rede ou servidor ao adicionar documento:', err);
          setError('Erro de rede ou servidor ao tentar adicionar o documento.');
        }
      } else {
        setError('Erro ao ler o arquivo selecionado.');
      }
    };

    reader.onerror = () => {
      setError('Erro ao ler o arquivo.');
    };
  };

  return (
    <div className="new-client-page-container">
      <h1 className="page-title">Adicionar Novo Documento</h1>
      <p className="page-description">Preencha os dados abaixo e faça upload de um documento.</p>

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

        {/* Área de Drag and Drop */}
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
            <p>Arquivo selecionado: <strong>{selectedFile.name}</strong></p>
          ) : (
            <p>Arraste e solte um arquivo aqui, ou clique para selecionar.</p>
          )}
          <input
            type="file"
            id="file_input"
            name="file_upload" // Nome do campo para o HTML, não relevante para o JSON de envio
            onChange={handleFileChange}
            style={{ display: 'none' }}
          />
        </div>
        {selectedFile && <p className="form-label" style={{ marginTop: '0.5rem' }}>Nome do Arquivo: {formData.nome_arquivo}</p>}


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