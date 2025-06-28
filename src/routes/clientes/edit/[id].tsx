// src/routes/clientes/edit/[id].tsx
import { useState, useEffect } from 'react';
import type { JSX } from 'react';
import { Link, useRouter } from 'tuono';

export default function EditClientPage(): JSX.Element {
  const router = useRouter();
  const id = router.pathname.split('/').pop();

  const [formData, setFormData] = useState({
    nome: '',
    email: '',
    telefone: '',
    endereco: '',
    tipoCliente: '', // 'fisica' or 'juridica' - will be set from fetched data
    cpf: '',
    cnpj: '',
  });
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [originalTipoCliente, setOriginalTipoCliente] = useState(''); // Store original type to handle type changes

  useEffect(() => {
    if (!id) {
      setIsLoading(false);
      return;
    }

    const fetchClientData = async () => {
      setIsLoading(true);
      try {
        const response = await fetch(`/api/clientes?id=${id}`);
        const text = await response.text();
        const data = JSON.parse(text);

        if (data.error) {
          setError(data.error);
        } else {
          setFormData({
            nome: data.nome,
            email: data.email,
            telefone: data.telefone,
            endereco: data.endereco,
            tipoCliente: data.cpf ? 'fisica' : (data.cnpj ? 'juridica' : ''), // Determine type
            cpf: data.cpf || '',
            cnpj: data.cnpj || '',
          });
          setOriginalTipoCliente(data.cpf ? 'fisica' : (data.cnpj ? 'juridica' : ''));
        }
      } catch (err) {
        console.error('Erro ao carregar dados do cliente para edição:', err);
        setError('Erro ao carregar dados do cliente para edição.');
      } finally {
        setIsLoading(false);
      }
    };

    fetchClientData();
  }, [id]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value } = e.target;
    setFormData(prev => ({ ...prev, [name]: value }));
  };

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setMessage('');
    setError('');

    // Basic validation
    if (!formData.nome || !formData.email || !formData.endereco || !formData.telefone) {
      setError('Por favor, preencha todos os campos obrigatórios (Nome, Email, Endereço, Telefone).');
      return;
    }

    if (formData.tipoCliente === 'fisica' && !formData.cpf) {
      setError('Por favor, insira o CPF para Pessoa Física.');
      return;
    }

    if (formData.tipoCliente === 'juridica' && !formData.cnpj) {
      setError('Por favor, insira o CNPJ para Pessoa Jurídica.');
      return;
    }

    const queryParams = new URLSearchParams({
      id: id!, // ID do cliente a ser atualizado
      nome: formData.nome,
      email: formData.email,
      telefone: formData.telefone,
      endereco: formData.endereco,
      tipoCliente: formData.tipoCliente,
      originalTipoCliente: originalTipoCliente, // Pass original type for backend logic
    });

    if (formData.tipoCliente === 'fisica') {
      queryParams.append('cpf', formData.cpf);
    } else {
      queryParams.append('cnpj', formData.cnpj);
    }

    try {
      const response = await fetch(`/api/clientes?${queryParams.toString()}`, {
        method: 'PUT', // Use PUT for updates
      });

      if (response.ok) {
        setMessage('Cliente atualizado com sucesso!');
        setTimeout(() => {
          router.push(`/clientes/${id}`); // Redirect to client details page
        }, 1500);
      } else {
        const errorText = await response.text();
        console.error('Erro ao atualizar cliente:', response.status, errorText);
        setError(`Erro ao atualizar cliente: ${response.status} - ${errorText || 'Erro desconhecido.'}`);
      }
    } catch (err) {
      console.error('Erro de rede ou servidor ao atualizar cliente:', err);
      setError('Erro de rede ou servidor ao tentar atualizar o cliente.');
    }
  };

  if (isLoading) {
    return <div className="loading-container"><h1>Carregando dados para edição...</h1></div>;
  }

  if (error && !message) { // Show error if no success message
    return <p className="error-message" style={{ margin: '2.5rem' }}>{error}</p>;
  }

  return (
    <div className="new-client-page-container">
      <h1 className="page-title">Editar Cliente</h1>
      <p className="page-description">Altere os dados do cliente e salve as modificações.</p>

      <form onSubmit={handleSubmit} className="client-form">
        <div className="form-group">
          <label className="form-label">Tipo de Cliente:</label>
          <div style={{ display: 'flex', gap: '1rem' }}>
            <label>
              <input
                type="radio"
                name="tipoCliente"
                value="fisica"
                checked={formData.tipoCliente === 'fisica'}
                onChange={handleChange}
              />{' '}
              Pessoa Física
            </label>
            <label>
              <input
                type="radio"
                name="tipoCliente"
                value="juridica"
                checked={formData.tipoCliente === 'juridica'}
                onChange={handleChange}
              />{' '}
              Pessoa Jurídica
            </label>
          </div>
        </div>

        <div className="form-group">
          <label htmlFor="nome" className="form-label">Nome do Cliente:</label>
          <input
            type="text"
            id="nome"
            name="nome"
            className="form-input"
            value={formData.nome}
            onChange={handleChange}
            required
          />
        </div>
        <div className="form-group">
          <label htmlFor="email" className="form-label">Email:</label>
          <input
            type="email"
            id="email"
            name="email"
            className="form-input"
            value={formData.email}
            onChange={handleChange}
            required
          />
        </div>
        <div className="form-group">
          <label htmlFor="telefone" className="form-label">Telefone:</label>
          <input
            type="tel"
            id="telefone"
            name="telefone"
            className="form-input"
            value={formData.telefone}
            onChange={handleChange}
            required
          />
        </div>
        <div className="form-group">
          <label htmlFor="endereco" className="form-label">Endereço:</label>
          <input
            type="text"
            id="endereco"
            name="endereco"
            className="form-input"
            value={formData.endereco}
            onChange={handleChange}
            required
          />
        </div>

        {formData.tipoCliente === 'fisica' && (
          <div className="form-group">
            <label htmlFor="cpf" className="form-label">CPF:</label>
            <input
              type="text"
              id="cpf"
              name="cpf"
              className="form-input"
              value={formData.cpf}
              onChange={handleChange}
              required={formData.tipoCliente === 'fisica'}
            />
          </div>
        )}

        {formData.tipoCliente === 'juridica' && (
          <div className="form-group">
            <label htmlFor="cnpj" className="form-label">CNPJ:</label>
            <input
              type="text"
              id="cnpj"
              name="cnpj"
              className="form-input"
              value={formData.cnpj}
              onChange={handleChange}
              required={formData.tipoCliente === 'juridica'}
            />
          </div>
        )}

        {message && <p className="success-message">{message}</p>}
        {error && <p className="error-message">{error}</p>}

        <div className="form-actions">
          <button type="submit" className="submit-button">
            Salvar Alterações
          </button>
          <Link href={`/clientes/${id}`} className="cancel-button">
            Cancelar
          </Link>
        </div>
      </form>
    </div>
  );
}