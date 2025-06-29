import { useState, useEffect } from 'react';
import type { JSX } from 'react';
import { Link, useRouter } from 'tuono';

// Interfaces para os dados de lookup (clientes, advogados, etc.)
interface LookupItem {
  id: string; // ID como string para consistência com HTML <select> value
  nome: string;
}

export default function NewCasePage(): JSX.Element {
  const router = useRouter();
  const [formData, setFormData] = useState({
    id_cliente: '',
    id_advogado: '',
    id_status: '',
    id_vara_judicial: '',
    id_categoria_caso: '',
    descricao: '',
    numero_processo: '',
    data_abertura: '',
    data_fechamento: '',
  });
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [isLoadingLookups, setIsLoadingLookups] = useState(true);

  // Estados para os dados dos dropdowns
  const [clientes, setClientes] = useState<LookupItem[]>([]);
  const [advogados, setAdvogados] = useState<LookupItem[]>([]);
  const [statusOptions, setStatusOptions] = useState<LookupItem[]>([]);
  const [varasJudiciais, setVarasJudiciais] = useState<LookupItem[]>([]);
  const [categoriasCaso, setCategoriasCaso] = useState<LookupItem[]>([]);

  useEffect(() => {
    const fetchLookups = async () => {
      setIsLoadingLookups(true);
      try {
        const fetchData = async (url: string) => {
          const res = await fetch(url);
          if (!res.ok) {
            const errorBody = await res.text();
            throw new Error(`Falha ao carregar ${url}: ${res.status} - ${errorBody}`);
          }
          return res.json();
        };

        // Fetch Clientes: Backend retorna { id_cliente: number, nome: string }
        const clientesData: { id_cliente: number, nome: string }[] = await fetchData('/api/clientes');
        setClientes(clientesData.map(c => ({ id: c.id_cliente.toString(), nome: c.nome })));

        // Fetch Advogados: Backend retorna { id: number, nome: string } (já formatado)
        const advogadosData: { id: number, nome: string }[] = await fetchData('/api/advogados');
        setAdvogados(advogadosData.map(a => ({ id: a.id.toString(), nome: a.nome })));

        // Fetch Status: Backend retorna { id: number, nome: string }
        const statusData: { id: number, nome: string }[] = await fetchData('/api/status');
        setStatusOptions(statusData.map(s => ({ id: s.id.toString(), nome: s.nome })));

        // Fetch Varas Judiciais: Backend retorna { id: number, nome: string }
        const varasData: { id: number, nome: string }[] = await fetchData('/api/varas_judiciais');
        setVarasJudiciais(varasData.map(v => ({ id: v.id.toString(), nome: v.nome })));

        // Fetch Categorias de Caso: Backend retorna { id: number, nome: string }
        const categoriasData: { id: number, nome: string }[] = await fetchData('/api/categorias_caso');
        setCategoriasCaso(categoriasData.map(c => ({ id: c.id.toString(), nome: c.nome })));


      } catch (err: any) { // Captura o erro tipado
        console.error('Erro ao carregar dados para dropdowns:', err);
        setError(`Erro ao carregar opções para os campos: ${err.message || 'Erro desconhecido.'} Verifique os endpoints de lookup.`);
      } finally {
        setIsLoadingLookups(false);
      }
    };

    fetchLookups();
  }, []);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
    const { name, value } = e.target;
    setFormData((prev: typeof formData) => ({ ...prev, [name]: value }));
  };

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setMessage('');
    setError('');

    if (!formData.id_cliente || !formData.id_advogado || !formData.id_status || !formData.data_abertura) {
      setError('Por favor, preencha todos os campos obrigatórios (Cliente, Advogado, Status, Data Abertura).');
      return;
    }

    const payload = {
      id_cliente: parseInt(formData.id_cliente),
      id_advogado: parseInt(formData.id_advogado),
      id_status: parseInt(formData.id_status),
      id_vara_judicial: formData.id_vara_judicial ? parseInt(formData.id_vara_judicial) : null,
      id_categoria_caso: formData.id_categoria_caso ? parseInt(formData.id_categoria_caso) : null,
      descricao: formData.descricao || null,
      numero_processo: formData.numero_processo || null,
      data_abertura: formData.data_abertura,
      data_fechamento: formData.data_fechamento || null,
    };

    try {
      const response = await fetch('/api/casos', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
      });

      if (response.ok) {
        setMessage('Caso jurídico criado com sucesso!');
        setFormData({
          id_cliente: '', id_advogado: '', id_status: '', id_vara_judicial: '',
          id_categoria_caso: '', descricao: '', numero_processo: '', data_abertura: '', data_fechamento: '',
        });
        setTimeout(() => {
          router.push('/casos');
        }, 1500);
      } else {
        const errorText = await response.text();
        console.error('Erro ao criar caso:', response.status, errorText);
        try {
          const errorJson = JSON.parse(errorText);
          setError(`Erro ao criar caso: ${response.status} - ${errorJson.error || 'Erro desconhecido.'}`);
        } catch {
          setError(`Erro ao criar caso: ${response.status} - ${errorText || 'Erro desconhecido.'}`);
        }
      }
    } catch (err) {
      console.error('Erro de rede ou servidor ao criar caso:', err);
      setError('Erro de rede ou servidor ao tentar criar o caso.');
    }
  };

  if (isLoadingLookups) {
    return <div className="loading-container"><h1>Carregando opções para o formulário...</h1></div>;
  }

  return (
    <div className="new-client-page-container">
      <h1 className="page-title">Adicionar Novo Caso Jurídico</h1>
      <p className="page-description">Preencha os dados abaixo para registrar um novo caso.</p>

      <form onSubmit={handleSubmit} className="client-form">
        <div className="form-group">
          <label htmlFor="id_cliente" className="form-label">Cliente:</label>
          <select
            id="id_cliente"
            name="id_cliente"
            className="form-input"
            value={formData.id_cliente}
            onChange={handleChange}
            required
          >
            <option value="">Selecione um cliente</option>
            {clientes.map(c => (
              <option key={c.id} value={c.id}>{c.nome}</option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label htmlFor="id_advogado" className="form-label">Advogado Responsável:</label>
          <select
            id="id_advogado"
            name="id_advogado"
            className="form-input"
            value={formData.id_advogado}
            onChange={handleChange}
            required
          >
            <option value="">Selecione um advogado</option>
            {advogados.map(a => (
              <option key={a.id} value={a.id}>{a.nome}</option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label htmlFor="id_status" className="form-label">Status do Caso:</label>
          <select
            id="id_status"
            name="id_status"
            className="form-input"
            value={formData.id_status}
            onChange={handleChange}
            required
          >
            <option value="">Selecione um status</option>
            {statusOptions.map(s => (
              <option key={s.id} value={s.id}>{s.nome}</option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label htmlFor="id_vara_judicial" className="form-label">Vara Judicial (Opcional):</label>
          <select
            id="id_vara_judicial"
            name="id_vara_judicial"
            className="form-input"
            value={formData.id_vara_judicial}
            onChange={handleChange}
          >
            <option value="">Nenhuma</option>
            {varasJudiciais.map(v => (
              <option key={v.id} value={v.id}>{v.nome}</option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label htmlFor="id_categoria_caso" className="form-label">Categoria do Caso (Opcional):</label>
          <select
            id="id_categoria_caso"
            name="id_categoria_caso"
            className="form-input"
            value={formData.id_categoria_caso}
            onChange={handleChange}
          >
            <option value="">Nenhuma</option>
            {categoriasCaso.map(cat => (
              <option key={cat.id} value={cat.id}>{cat.nome}</option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label htmlFor="descricao" className="form-label">Descrição (Opcional):</label>
          <textarea
            id="descricao"
            name="descricao"
            className="form-input"
            value={formData.descricao}
            onChange={handleChange}
            rows={3}
          />
        </div>

        <div className="form-group">
          <label htmlFor="numero_processo" className="form-label">Número do Processo (Opcional):</label>
          <input
            type="text"
            id="numero_processo"
            name="numero_processo"
            className="form-input"
            value={formData.numero_processo}
            onChange={handleChange}
          />
        </div>

        <div className="form-group">
          <label htmlFor="data_abertura" className="form-label">Data de Abertura:</label>
          <input
            type="date"
            id="data_abertura"
            name="data_abertura"
            className="form-input"
            value={formData.data_abertura}
            onChange={handleChange}
            required
          />
        </div>

        <div className="form-group">
          <label htmlFor="data_fechamento" className="form-label">Data de Fechamento (Opcional):</label>
          <input
            type="date"
            id="data_fechamento"
            name="data_fechamento"
            className="form-input"
            value={formData.data_fechamento}
            onChange={handleChange}
          />
        </div>

        {message && <p className="success-message">{message}</p>}
        {error && <p className="error-message">{error}</p>}

        <div className="form-actions">
          <button type="submit" className="submit-button">
            Criar Caso
          </button>
          <Link href="/casos" className="cancel-button">
            Cancelar
          </Link>
        </div>
      </form>
    </div>
  );
}