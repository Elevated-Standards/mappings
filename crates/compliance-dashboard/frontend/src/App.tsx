import React, { useState, useEffect } from 'react';
import { Layout, Container, Grid, GridItem } from './components/Layout';
import { Header, Sidebar, Navigation, MenuToggle, Breadcrumb } from './components/Navigation';
import { ControlCard, ControlGrid } from './components/UI/ControlCard';
import { StatusIndicator } from './components/UI/StatusIndicator';
import { ProgressBar } from './components/UI/ProgressBar';
import { apiService, mockData, dashboardUtils } from './services/api';
import type { Control, Framework, ImplementationStatus } from './types';
import type { DashboardData } from './services/api';
import './App.css';

// Navigation items configuration

const navigationItems = [
  {
    id: 'dashboard',
    label: 'Dashboard',
    href: '/',
    icon: '📊',
    active: true
  },
  {
    id: 'controls',
    label: 'Controls',
    href: '/controls',
    icon: '🛡️'
  },
  {
    id: 'frameworks',
    label: 'Frameworks',
    href: '/frameworks',
    icon: '📋'
  },
  {
    id: 'reports',
    label: 'Reports',
    href: '/reports',
    icon: '📈'
  },
  {
    id: 'settings',
    label: 'Settings',
    href: '/settings',
    icon: '⚙️'
  }
];

function App() {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [selectedFramework, setSelectedFramework] = useState('nist-800-53');
  const [dashboardData, setDashboardData] = useState<DashboardData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<'connected' | 'disconnected' | 'connecting'>('connecting');

  // Load dashboard data on component mount
  useEffect(() => {
    loadDashboardData();

    // Set up periodic refresh
    const interval = setInterval(loadDashboardData, 30000); // Refresh every 30 seconds

    return () => clearInterval(interval);
  }, []);

  const loadDashboardData = async () => {
    try {
      setConnectionStatus('connecting');

      // Try to load from API first
      const response = await apiService.getDashboard();

      if (response.data) {
        setDashboardData(response.data);
        setConnectionStatus('connected');
        setError(null);
      } else {
        // Fall back to mock data if API is not available
        console.warn('API not available, using mock data:', response.error);
        setDashboardData(mockData.dashboard);
        setConnectionStatus('disconnected');
        setError('API not available - using demo data');
      }
    } catch (err) {
      console.error('Failed to load dashboard data:', err);
      setDashboardData(mockData.dashboard);
      setConnectionStatus('disconnected');
      setError('Failed to connect to API - using demo data');
    } finally {
      setLoading(false);
    }
  };

  const handleControlClick = (control: Control) => {
    console.log('Control clicked:', control);
  };

  const handleControlStatusUpdate = async (controlId: string, newStatus: ImplementationStatus) => {
    try {
      const response = await apiService.updateControlStatus(controlId, newStatus);
      if (response.data?.success) {
        // Refresh dashboard data after successful update
        await loadDashboardData();
      } else {
        console.error('Failed to update control status:', response.error);
      }
    } catch (err) {
      console.error('Error updating control status:', err);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <div className="text-2xl font-semibold mb-2">Loading Dashboard...</div>
          <div className="text-gray-600">Connecting to compliance data</div>
        </div>
      </div>
    );
  }

  if (!dashboardData) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <div className="text-2xl font-semibold mb-2 text-red-600">Failed to Load Dashboard</div>
          <div className="text-gray-600 mb-4">Unable to connect to the compliance dashboard</div>
          <button
            onClick={loadDashboardData}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  const { overview } = dashboardData;
  const statusCounts = {
    implemented: overview.implemented_controls,
    'in-progress': overview.in_progress_controls,
    'not-implemented': overview.not_implemented_controls,
    'not-applicable': 0 // Not provided in current API
  };
  const totalControls = overview.total_controls;
  const implementedPercentage = overview.implementation_percentage;

  return (
    <Layout>
      <Header>
        <div className="flex items-center justify-between w-full">
          <div className="flex items-center gap-4">
            <MenuToggle
              isOpen={sidebarOpen}
              onToggle={() => setSidebarOpen(!sidebarOpen)}
              className="lg:hidden"
            />
            <h1 className="text-xl font-bold text-gray-900">
              FedRAMP Compliance Dashboard
            </h1>
          </div>
          <div className="flex items-center gap-4">
            <StatusIndicator status={connectionStatus} label="System Status" />
            <div className="text-sm text-gray-600">
              Last updated: {dashboardUtils.formatDate(overview.last_updated)}
            </div>
            {error && (
              <div className="text-sm text-yellow-600 bg-yellow-50 px-2 py-1 rounded">
                {error}
              </div>
            )}
          </div>
        </div>
      </Header>

      <div className="flex flex-1">
        <Sidebar
          isOpen={sidebarOpen}
          onClose={() => setSidebarOpen(false)}
          width="md"
        >
          <div className="p-4">
            <h2 className="text-lg font-semibold mb-4">Navigation</h2>
            <Navigation
              items={navigationItems}
              variant="vertical"
              onItemClick={(item) => console.log('Navigation:', item)}
            />
          </div>
        </Sidebar>

        <main className="flex-1 p-6">
          <Container size="full">
            <Breadcrumb
              items={[
                { label: 'Home', href: '/' },
                { label: 'Dashboard' }
              ]}
              className="mb-6"
            />

            <div className="mb-8">
              <h2 className="text-2xl font-bold mb-4">Compliance Overview</h2>
              <Grid cols={{ mobile: 1, tablet: 2, desktop: 4 }} gap="md">
                <GridItem>
                  <div className="bg-white p-6 rounded-lg shadow-sm border">
                    <h3 className="text-sm font-medium text-gray-600 mb-2">Total Controls</h3>
                    <div className="text-3xl font-bold text-gray-900">{totalControls}</div>
                  </div>
                </GridItem>
                <GridItem>
                  <div className="bg-white p-6 rounded-lg shadow-sm border">
                    <h3 className="text-sm font-medium text-gray-600 mb-2">Implemented</h3>
                    <div className="text-3xl font-bold text-green-600">{statusCounts.implemented}</div>
                  </div>
                </GridItem>
                <GridItem>
                  <div className="bg-white p-6 rounded-lg shadow-sm border">
                    <h3 className="text-sm font-medium text-gray-600 mb-2">In Progress</h3>
                    <div className="text-3xl font-bold text-yellow-600">{statusCounts['in-progress']}</div>
                  </div>
                </GridItem>
                <GridItem>
                  <div className="bg-white p-6 rounded-lg shadow-sm border">
                    <h3 className="text-sm font-medium text-gray-600 mb-2">Not Implemented</h3>
                    <div className="text-3xl font-bold text-red-600">{statusCounts['not-implemented']}</div>
                  </div>
                </GridItem>
              </Grid>
            </div>

            <div className="mb-8">
              <h3 className="text-xl font-semibold mb-4">Implementation Progress</h3>
              <div className="bg-white p-6 rounded-lg shadow-sm border">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-medium text-gray-600">Overall Progress</span>
                  <span className="text-sm font-medium text-gray-900">{implementedPercentage.toFixed(1)}%</span>
                </div>
                <ProgressBar
                  value={implementedPercentage}
                  max={100}
                  variant="success"
                  size="lg"
                />
              </div>
            </div>

            <div>
              <h3 className="text-xl font-semibold mb-4">Recent Controls</h3>
              <ControlGrid
                controls={overview.recent_updates}
                onControlClick={handleControlClick}
                className="mb-6"
              />
            </div>
          </Container>
        </main>
      </div>
    </Layout>
  );
}

export default App;
