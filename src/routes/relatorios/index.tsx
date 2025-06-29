import { useState, useEffect } from 'react';
import type { JSX } from 'react';
import { useRouter } from 'tuono';
import type { TuonoRouteProps } from 'tuono';

import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

interface ReportDataItem {
  cliente_nome: string;
  numero_processo?: string;
  total_documentos: number;
}

interface RelatoriosPageProps {
  report_data_docs_clientes_casos: ReportDataItem[];
}

export default function RelatoriosPage({ data, isLoading: propIsLoading }: TuonoRouteProps<RelatoriosPageProps>): JSX.Element {
  // Inicializa o estado com os dados pré-renderizados, se disponíveis
  const [reportData, setReportData] = useState<ReportDataItem[]>(data?.report_data_docs_clientes_casos || []);
  const [error, setError] = useState<string | null>(null);
  const [isLoadingPage, setIsLoadingPage] = useState(propIsLoading);

  useEffect(() => {
    // Esta lógica de fetch client-side só é necessária se os dados não foram pré-renderizados
    // (ex: durante o desenvolvimento ou se houver problemas com o handler de pré-renderização)
    const fetchDataClient = async () => {
      setIsLoadingPage(true);
      try {
        if (!data?.report_data_docs_clientes_casos) {
          // AQUI VOCÊ PODE ADICIONAR UM CONSOLE.WARN OU TRATAMENTO SE REALMENTE ESPERAR UM FALLBACK DE API
          // console.warn("Dados do relatório não pré-renderizados. O relatório pode estar vazio.");
        }
      } catch (err: any) {
        console.error("Erro ao carregar dados do relatório (cliente-side fallback):", err);
        setError(`Erro ao carregar relatório: ${err.message || 'Erro desconhecido.'}`);
      } finally {
        setIsLoadingPage(false);
      }
    };

    if (data?.report_data_docs_clientes_casos) {
      setReportData(data.report_data_docs_clientes_casos);
      setIsLoadingPage(false);
    } else {
      fetchDataClient(); 
    }
  }, [data]);

  if (isLoadingPage) {
    return <div className="loading-container"><h1>Carregando relatório...</h1></div>;
  }

  if (error) {
    return <p className="error-message" style={{ margin: '2.5rem' }}>{error}</p>;
  }

  // Preparação dos dados para o Recharts
  // Cada barra representará um (Cliente, Número do Processo)
  const chartData = reportData.map(item => ({
    name: `${item.cliente_nome} (${item.numero_processo || 'Sem Processo'})`, // Combina cliente e processo
    "Total de Documentos": item.total_documentos, // Série para o gráfico
  }));


  return (
    <div className="new-client-page-container"> {/* Reutilizando o container */}
      <h1 className="page-title">Relatório: Documentos por Cliente e Caso</h1>
      <p className="page-description">Quantidade de documentos vinculados a cada cliente e seus respectivos casos.</p>

      {reportData.length === 0 ? (
        <p className="no-results-message">Nenhum dado disponível para o relatório. Verifique se há documentos, casos ou clientes no banco de dados.</p>
      ) : (
        <div style={{ width: '100%', height: 400 }}>
          <ResponsiveContainer>
            <BarChart
              data={chartData}
              margin={{ top: 20, right: 30, left: 20, bottom: 5 }}
            >
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="name" interval={0} angle={-30} textAnchor="end" height={80} />
              <YAxis />
              <Tooltip />
              <Legend />
              <Bar dataKey="Total de Documentos" fill="#007bff" />
            </BarChart>
          </ResponsiveContainer>
        </div>
      )}
    </div>
  );
}