//clientes/[id].tsx
import { useRouter, Link } from 'tuono' // Import Link for navigation
import { useEffect, useState } from 'react'
import type { JSX } from 'react' // Import JSX

export default function ClientePage(): JSX.Element {
  const router = useRouter()
  const [cliente, setCliente] = useState<any>(null)
  const [erro, setErro] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState<boolean>(true); // Add loading state

  const id = router.pathname.split('/').pop()

  useEffect(() => {
    if (!id) {
      setIsLoading(false);
      return;
    }

    setIsLoading(true); // Set loading to true when fetching
    console.log("Buscando cliente com ID:", id)
    fetch(`/api/clientes?id=${id}`, {
      method: 'GET', 
    })
      .then(async (res) => {
        const text = await res.text()
        console.log("Resposta bruta do servidor:", text)

        try {
          const data = JSON.parse(text)
          if (data.error) {
            setErro(data.error)
          } else {
            setCliente(data)
          }
        } catch (e) {
          setErro("Resposta inválida do servidor. Não é um JSON válido.")
        } finally {
            setIsLoading(false); // Set loading to false after fetch
        }
      })
      .catch((err) => {
        setErro(`Erro ao buscar cliente: ${err.message}`)
        setIsLoading(false); // Set loading to false on error
      })
  }, [id])

  if (isLoading) {
    return <div className="loading-container"><h1>Carregando detalhes do cliente...</h1></div>;
  }

  if (erro) return <p className="error-message" style={{ margin: '2.5rem' }}>{erro}</p>
  if (!cliente) return <p className="no-results-message" style={{ margin: '2.5rem' }}>Cliente não encontrado.</p>

  return (
    <div className="new-client-page-container"> {/* Reusing form container style */}
      <h1 className="page-title">Detalhes do Cliente</h1>
      <p className="page-description">Informações completas sobre o cliente.</p>

      <div className="client-form" style={{ gap: '1rem' }}> {/* Reusing form styling for display */}
        <div className="form-group">
          <label className="form-label">ID:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{cliente.id_cliente}</p>
        </div>
        <div className="form-group">
          <label className="form-label">Nome:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{cliente.nome}</p>
        </div>
        <div className="form-group">
          <label className="form-label">Email:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{cliente.email}</p>
        </div>
        <div className="form-group">
          <label className="form-label">Telefone:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{cliente.telefone}</p>
        </div>
        <div className="form-group">
          <label className="form-label">Endereço:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{cliente.endereco}</p>
        </div>
        <div className="form-group">
          <label className="form-label">Data de Cadastro:</label>
          <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{cliente.data_cadastro ? new Date(cliente.data_cadastro).toLocaleDateString('pt-BR') : 'Não informado'}</p>
        </div>
        {cliente.cpf && (
          <div className="form-group">
            <label className="form-label">CPF:</label>
            <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{cliente.cpf}</p>
          </div>
        )}
        {cliente.cnpj && (
          <div className="form-group">
            <label className="form-label">CNPJ:</label>
            <p className="form-input" style={{ backgroundColor: '#f0f0f0', border: '1px solid #e0e0e0' }}>{cliente.cnpj}</p>
          </div>
        )}
      </div>

      <div className="form-actions" style={{ justifyContent: 'flex-start' }}> {/* Align buttons to the left */}
        <Link href={`/clientes/edit/${cliente.id_cliente}`} className="submit-button" style={{ backgroundColor: '#ffc107', color: '#333' }}>
          Editar Cliente
        </Link>
        <Link href="/clientes" className="cancel-button">
          Voltar para Clientes
        </Link>
      </div>
    </div>
  )
}