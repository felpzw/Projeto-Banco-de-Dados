import { useState } from 'react';
import type { JSX } from 'react';
import { Link, useRouter } from 'tuono';

export default function NewClientPage(): JSX.Element {
  const router = useRouter();
  const [formData, setFormData] = useState({
    nome: '',
    email: '',
    telefone: '',
    endereco: '',
    tipoCliente: 'fisica', // 'fisica' or 'juridica'
    cpf: '',
    cnpj: '',
  });
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

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

    // Construct query parameters for the POST request
    const queryParams = new URLSearchParams({
      nome: formData.nome,
      email: formData.email,
      telefone: formData.telefone,
      endereco: formData.endereco,
      tipoCliente: formData.tipoCliente, // Send client type
    });

    if (formData.tipoCliente === 'fisica') {
      queryParams.append('cpf', formData.cpf);
    } else {
      queryParams.append('cnpj', formData.cnpj);
    }

    try {
      // Send data to the backend via POST request to /api/clientes
      const response = await fetch(`/api/clientes?${queryParams.toString()}`, {
        method: 'POST',
      });

      if (response.ok) {
        setMessage('Cliente adicionado com sucesso!');
        // Clear form after successful submission
        setFormData({
          nome: '',
          email: '',
          telefone: '',
          endereco: '',
          tipoCliente: 'fisica',
          cpf: '',
          cnpj: '',
        });
        // Redirect after a small delay
        setTimeout(() => {
          router.push('/clientes');
        }, 1500);
      } else {
        const errorText = await response.text(); // Get raw error text
        console.error('Erro ao adicionar cliente:', response.status, errorText);
        setError(`Erro ao adicionar cliente: ${response.status} - ${errorText || 'Erro desconhecido.'}`);
      }
    } catch (err) {
      console.error('Erro de rede ou servidor ao adicionar cliente:', err);
      setError('Erro de rede ou servidor ao tentar adicionar o cliente.');
    }
  };

  return (
    <div className="new-client-page-container">
      <h1 className="page-title">Adicionar Novo Cliente</h1>
      <p className="page-description">Preencha os dados abaixo para cadastrar um novo cliente no sistema LawIA.</p>

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
            Adicionar Cliente
          </button>
          <Link href="/clientes" className="cancel-button">
            Cancelar
          </Link>
        </div>
      </form>
    </div>
  );
}
