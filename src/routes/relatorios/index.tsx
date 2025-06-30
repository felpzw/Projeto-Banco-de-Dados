import { useState, useEffect } from 'react';
import type { JSX } from 'react';
import { useRouter } from 'tuono';
import type { TuonoRouteProps } from 'tuono';

// Importa componentes do Recharts
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

// Interfaces para os dados do relatório "Documentos por Cliente e Caso"
interface ReportDataItemDocsClientesCasos {
  cliente_nome: string;
  numero_processo?: string;
  total_documentos: number;
}

// Interfaces para os dados do relatório "Total de Casos por Advogado e Status"
interface ReportDataItemCasosAdvogadoStatus {
  advogado_nome: string;
  status_descricao: string;
  total_casos: number;
}

// Interfaces para os dados do relatório "Total de Audiências por Cliente e Advogado"
interface ReportDataItemAudienciasClienteAdvogado {
  cliente_nome: string;
  advogado_nome: string;
  total_audiencias: number;
}


// Propriedades da Página de Relatórios (Reflete as props do backend)
interface RelatoriosPageProps {
  report_data_docs_clientes_casos: ReportDataItemDocsClientesCasos[];
  report_data_casos_advogado_status: ReportDataItemCasosAdvogadoStatus[];
  report_data_audiencias_cliente_advogado: ReportDataItemAudienciasClienteAdvogado[];
}

export default function RelatoriosPage({ data, isLoading: propIsLoading }: TuonoRouteProps<RelatoriosPageProps>): JSX.Element {
  const router = useRouter();
  const [reportDataDocsClientesCasos, setReportDataDocsClientesCasos] = useState<ReportDataItemDocsClientesCasos[]>(data?.report_data_docs_clientes_casos || []);
  const [reportDataCasosAdvogadoStatus, setReportDataCasosAdvogadoStatus] = useState<ReportDataItemCasosAdvogadoStatus[]>(data?.report_data_casos_advogado_status || []);
  const [reportDataAudienciasClienteAdvogado, setReportDataAudienciasClienteAdvogado] = useState<ReportDataItemAudienciasClienteAdvogado[]>(data?.report_data_audiencias_cliente_advogado || []);
  
  const [error, setError] = useState<string | null>(null);
  const [isLoadingPage, setIsLoadingPage] = useState(propIsLoading);

  useEffect(() => {
    if (data?.report_data_docs_clientes_casos && data?.report_data_casos_advogado_status && data?.report_data_audiencias_cliente_advogado) {
      setReportDataDocsClientesCasos(data.report_data_docs_clientes_casos);
      setReportDataCasosAdvogadoStatus(data.report_data_casos_advogado_status);
      setReportDataAudienciasClienteAdvogado(data.report_data_audiencias_cliente_advogado);
      setIsLoadingPage(false);
    } else {
      console.warn("Dados do relatório não pré-renderizados. O relatório pode estar vazio.");
      setIsLoadingPage(false);
    }
  }, [data]);

  if (isLoadingPage) {
    return <div className="loading-container"><h1>Carregando relatórios...</h1></div>;
  }

  if (error) {
    return <p className="error-message" style={{ margin: '2.5rem' }}>{error}</p>;
  }

  // --- Preparação dos dados para o Recharts (Report 1: Documentos por Cliente e Caso) ---
  const chartDataDocsClientesCasos = reportDataDocsClientesCasos.map(item => ({
    name: `${item.cliente_nome} (${item.numero_processo || 'Sem Processo'})`,
    "Total de Documentos": item.total_documentos,
  }));

  // --- Preparação dos dados para o Recharts (Report 2: Total de Casos por Advogado e Status) ---
  const chartDataCasosAdvogadoStatus = reportDataCasosAdvogadoStatus.map(item => ({
    name: `${item.advogado_nome} (${item.status_descricao})`,
    "Total de Casos": item.total_casos,
  }));

  // --- Preparação dos dados para o Recharts (Report 3: Total de Audiências por Cliente e Advogado) ---
  const chartDataAudienciasClienteAdvogado = reportDataAudienciasClienteAdvogado.map(item => ({
    name: `${item.cliente_nome} (${item.advogado_nome})`,
    "Total de Audiências": item.total_audiencias,
  }));


  return (
    <div className="new-client-page-container">
      <h1 className="page-title">Relatórios Gerenciais</h1>
      <p className="page-description">Visões detalhadas sobre os dados jurídicos da plataforma.</p>

      {/* Relatório 1: Documentos por Cliente e Caso */}
      <h2 style={{ fontSize: '1.8rem', fontWeight: 600, color: 'var(--dark-text)', marginTop: '2rem', marginBottom: '1.5rem', textAlign: 'center' }}>
        Documentos por Cliente e Caso
      </h2>
      {reportDataDocsClientesCasos.length === 0 ? (
        <p className="no-results-message">Nenhum dado disponível para este relatório. Verifique documentos, casos e clientes.</p>
      ) : (
        <div style={{ width: '100%', height: 600 }}> {/* AUMENTADO A ALTURA */}
          <ResponsiveContainer>
            <BarChart
              data={chartDataDocsClientesCasos}
              margin={{ top: 20, right: 30, left: 20, bottom: 5 }}
            >
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="name" interval={0} angle={-45} textAnchor="end" height={150} /> {/* AUMENTADO ALTURA E ROTACIONADO MAIS */}
              <YAxis />
              <Tooltip />
              <Legend />
              <Bar dataKey="Total de Documentos" fill="#007bff" />
            </BarChart>
          </ResponsiveContainer>
        </div>
      )}

      {/* Relatório 2: Total de Casos por Advogado e Status */}
      <h2 style={{ fontSize: '1.8rem', fontWeight: 600, color: 'var(--dark-text)', marginTop: '3rem', marginBottom: '1.5rem', textAlign: 'center' }}>
        Casos por Advogado e Status
      </h2>
      {reportDataCasosAdvogadoStatus.length === 0 ? (
        <p className="no-results-message">Nenhum dado disponível para este relatório. Verifique casos, advogados e status.</p>
      ) : (
        <div style={{ width: '100%', height: 600 }}> {/* AUMENTADO A ALTURA */}
          <ResponsiveContainer>
            <BarChart
              data={chartDataCasosAdvogadoStatus}
              margin={{ top: 20, right: 30, left: 20, bottom: 5 }}
            >
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="name" interval={0} angle={-45} textAnchor="end" height={150} /> {/* AUMENTADO ALTURA E ROTACIONADO MAIS */}
              <YAxis />
              <Tooltip />
              <Legend />
              <Bar dataKey="Total de Casos" fill="#28a745" /> {/* Outra cor */}
            </BarChart>
          </ResponsiveContainer>
        </div>
      )}

      {/* Relatório 3: Total de Audiências por Cliente e Advogado */}
      <h2 style={{ fontSize: '1.8rem', fontWeight: 600, color: 'var(--dark-text)', marginTop: '3rem', marginBottom: '1.5rem', textAlign: 'center' }}>
        Audiências por Cliente e Advogado
      </h2>
      {reportDataAudienciasClienteAdvogado.length === 0 ? (
        <p className="no-results-message">Nenhum dado disponível para este relatório. Verifique audiências, casos, clientes e advogados.</p>
      ) : (
        <div style={{ width: '100%', height: 600 }}> {/* AUMENTADO A ALTURA */}
          <ResponsiveContainer>
            <BarChart
              data={chartDataAudienciasClienteAdvogado}
              margin={{ top: 20, right: 30, left: 20, bottom: 5 }}
            >
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="name" interval={0} angle={-45} textAnchor="end" height={150} /> {/* AUMENTADO ALTURA E ROTACIONADO MAIS */}
              <YAxis />
              <Tooltip />
              <Legend />
              <Bar dataKey="Total de Audiências" fill="#ffc107" /> {/* Outra cor */}
            </BarChart>
          </ResponsiveContainer>
        </div>
      )}

    </div>
  );
}