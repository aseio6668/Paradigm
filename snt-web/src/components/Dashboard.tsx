import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import styled from 'styled-components';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';
import { SNT, Keeper, NetworkStats } from '../types/snt';
import { SNTCard } from './SNTCard';
import { NetworkVisualization } from './NetworkVisualization';
import { mockKeepers, mockSNTs, mockEvents } from '../data/mockData';
import { getSNTTypeName, formatFileSize, getElementColor } from '../utils/sntHelpers';

const DashboardContainer = styled.div`
  min-height: 100vh;
  padding: 20px;
  background: linear-gradient(135deg, #0f0f23 0%, #1a1a2e 50%, #16213e 100%);
`;

const Header = styled(motion.header)`
  text-align: center;
  margin-bottom: 40px;
`;

const Title = styled.h1`
  font-size: 3rem;
  background: linear-gradient(45deg, #64b3f4, #c2e9fb, #a18cd1, #fbc2eb);
  background-size: 400% 400%;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: gradientShift 4s ease-in-out infinite;
  margin: 0;
  
  @keyframes gradientShift {
    0%, 100% { background-position: 0% 50%; }
    50% { background-position: 100% 50%; }
  }
`;

const Subtitle = styled.p`
  font-size: 1.2rem;
  color: #cccccc;
  margin-top: 8px;
  opacity: 0.9;
`;

const TabContainer = styled.div`
  display: flex;
  justify-content: center;
  margin-bottom: 40px;
  gap: 8px;
`;

const Tab = styled(motion.button)<{ $active: boolean }>`
  background: ${props => props.$active 
    ? 'linear-gradient(45deg, #4facfe, #00f2fe)' 
    : 'rgba(255, 255, 255, 0.1)'};
  border: 2px solid ${props => props.$active ? 'transparent' : 'rgba(255, 255, 255, 0.2)'};
  color: #ffffff;
  padding: 12px 24px;
  border-radius: 25px;
  cursor: pointer;
  font-size: 16px;
  font-weight: bold;
  transition: all 0.3s ease;
  
  &:hover {
    background: ${props => props.$active 
      ? 'linear-gradient(45deg, #4facfe, #00f2fe)' 
      : 'rgba(255, 255, 255, 0.2)'};
  }
`;

const ContentArea = styled(motion.div)`
  max-width: 1400px;
  margin: 0 auto;
`;

const StatsGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 40px;
`;

const StatCard = styled(motion.div)<{ $color: string }>`
  background: rgba(255, 255, 255, 0.05);
  border: 2px solid rgba(${props => hexToRgb(props.$color)}, 0.3);
  border-radius: 16px;
  padding: 24px;
  text-align: center;
  backdrop-filter: blur(10px);
  
  &:hover {
    border-color: ${props => props.$color};
    box-shadow: 0 8px 32px rgba(${props => hexToRgb(props.$color)}, 0.2);
  }
`;

const StatValue = styled.div<{ $color: string }>`
  font-size: 2.5rem;
  font-weight: bold;
  color: ${props => props.$color};
  margin-bottom: 8px;
`;

const StatLabel = styled.div`
  font-size: 1rem;
  color: #cccccc;
  opacity: 0.9;
`;

const SectionTitle = styled.h2`
  font-size: 2rem;
  color: #ffffff;
  margin-bottom: 24px;
  text-align: center;
`;

const SNTGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 24px;
  margin-bottom: 40px;
`;

const ChartContainer = styled.div`
  background: rgba(255, 255, 255, 0.05);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  padding: 24px;
  margin-bottom: 40px;
  backdrop-filter: blur(10px);
`;

const EventsContainer = styled.div`
  background: rgba(255, 255, 255, 0.05);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  padding: 24px;
  backdrop-filter: blur(10px);
  max-height: 400px;
  overflow-y: auto;
`;

const EventItem = styled(motion.div)`
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.05);
  margin-bottom: 8px;
  border-left: 4px solid #4facfe;
`;

const EventTime = styled.span`
  color: #888888;
  font-size: 12px;
  min-width: 60px;
`;

const EventDetails = styled.span`
  color: #ffffff;
  flex: 1;
`;

function hexToRgb(hex: string): string {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) return '255, 255, 255';
  
  return `${parseInt(result[1], 16)}, ${parseInt(result[2], 16)}, ${parseInt(result[3], 16)}`;
}

export const Dashboard: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'overview' | 'snts' | 'network' | 'analytics'>('overview');

  // Calculate network stats
  const networkStats: NetworkStats = {
    total_snts: mockSNTs.length,
    type_distribution: mockSNTs.reduce((acc, snt) => {
      const typeName = getSNTTypeName(snt.token_type);
      acc[typeName] = (acc[typeName] || 0) + 1;
      return acc;
    }, {} as Record<string, number>),
    average_evolution_level: mockSNTs.reduce((sum, snt) => sum + snt.evolution_level, 0) / mockSNTs.length,
    unique_holders: new Set(mockSNTs.map(snt => snt.holder)).size,
    active_keepers: mockKeepers.length,
    total_sigils: mockKeepers.reduce((sum, keeper) => sum + keeper.sigils_hosted.length, 0),
    total_storage_used: mockKeepers.reduce((sum, keeper) => sum + keeper.used_storage, 0),
    network_utilization: mockKeepers.reduce((sum, keeper) => sum + (keeper.used_storage / keeper.capacity), 0) / mockKeepers.length * 100,
    recent_events: mockEvents.length
  };

  // Prepare chart data
  const typeDistributionData = Object.entries(networkStats.type_distribution).map(([type, count]) => ({
    type: type.replace(/[🛡️📦📜⚗️🎨🤝]/g, '').trim(),
    count,
    color: getElementColor('Fire') // Could map to actual element colors
  }));

  const keeperStatsData = mockKeepers.map(keeper => ({
    name: keeper.name,
    reputation: keeper.reputation,
    snts: mockSNTs.filter(snt => snt.holder === keeper.keeper_id).length,
    storage: keeper.used_storage / (1024 * 1024)
  }));

  const formatTimeAgo = (timestamp: number) => {
    const seconds = Math.floor((Date.now() - timestamp) / 1000);
    if (seconds < 60) return `${seconds}s ago`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  };

  return (
    <DashboardContainer>
      <Header
        initial={{ opacity: 0, y: -50 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <Title>🔮 Paradigm SNT Network</Title>
        <Subtitle>Revolutionary Symbolic Network Tokens - Living Functionality, Not Dead Commodities</Subtitle>
      </Header>

      <TabContainer>
        {[
          { id: 'overview', label: '📊 Network Overview' },
          { id: 'snts', label: '🎯 SNT Collection' },
          { id: 'network', label: '🌐 Network Visualization' },
          { id: 'analytics', label: '📈 Analytics Dashboard' }
        ].map((tab) => (
          <Tab
            key={tab.id}
            $active={activeTab === tab.id}
            onClick={() => setActiveTab(tab.id as any)}
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
          >
            {tab.label}
          </Tab>
        ))}
      </TabContainer>

      <ContentArea>
        <AnimatePresence mode="wait">
          {activeTab === 'overview' && (
            <motion.div
              key="overview"
              initial={{ opacity: 0, x: -100 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 100 }}
              transition={{ duration: 0.3 }}
            >
              <StatsGrid>
                {[
                  { label: '🔗 Active Keepers', value: networkStats.active_keepers, color: '#4facfe' },
                  { label: '🎯 Total SNTs', value: networkStats.total_snts, color: '#00f2fe' },
                  { label: '👥 Unique Holders', value: networkStats.unique_holders, color: '#4facfe' },
                  { label: '📦 Total Sigils', value: networkStats.total_sigils, color: '#00f2fe' },
                  { label: '⬆️ Avg Evolution Level', value: networkStats.average_evolution_level.toFixed(1), color: '#4facfe' },
                  { label: '💾 Storage Used', value: formatFileSize(networkStats.total_storage_used), color: '#00f2fe' },
                  { label: '📈 Network Utilization', value: `${networkStats.network_utilization.toFixed(1)}%`, color: '#4facfe' },
                  { label: '⚡ Recent Events', value: networkStats.recent_events, color: '#00f2fe' }
                ].map((stat, index) => (
                  <StatCard
                    key={stat.label}
                    $color={stat.color}
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.5, delay: index * 0.1 }}
                    whileHover={{ y: -4 }}
                  >
                    <StatValue $color={stat.color}>{stat.value}</StatValue>
                    <StatLabel>{stat.label}</StatLabel>
                  </StatCard>
                ))}
              </StatsGrid>

              <SectionTitle>⚡ Recent Network Activity</SectionTitle>
              <EventsContainer>
                {mockEvents.slice(0, 10).map((event, index) => (
                  <EventItem
                    key={`${event.timestamp}-${index}`}
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.3, delay: index * 0.05 }}
                  >
                    <EventTime>{formatTimeAgo(event.timestamp)}</EventTime>
                    <EventDetails>
                      <strong>{event.event_type.replace('_', ' ')}</strong> - {event.details}
                    </EventDetails>
                  </EventItem>
                ))}
              </EventsContainer>
            </motion.div>
          )}

          {activeTab === 'snts' && (
            <motion.div
              key="snts"
              initial={{ opacity: 0, x: -100 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 100 }}
              transition={{ duration: 0.3 }}
            >
              <SectionTitle>🎯 Active SNT Collection</SectionTitle>
              <SNTGrid>
                {mockSNTs.map((snt) => (
                  <SNTCard key={snt.snt_id} snt={snt} />
                ))}
              </SNTGrid>
            </motion.div>
          )}

          {activeTab === 'network' && (
            <motion.div
              key="network"
              initial={{ opacity: 0, x: -100 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 100 }}
              transition={{ duration: 0.3 }}
            >
              <SectionTitle>🌐 Living Network Visualization</SectionTitle>
              <NetworkVisualization keepers={mockKeepers} snts={mockSNTs} />
            </motion.div>
          )}

          {activeTab === 'analytics' && (
            <motion.div
              key="analytics"
              initial={{ opacity: 0, x: -100 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 100 }}
              transition={{ duration: 0.3 }}
            >
              <SectionTitle>📈 Network Analytics</SectionTitle>
              
              <ChartContainer>
                <h3 style={{ color: '#ffffff', marginBottom: '20px' }}>SNT Type Distribution</h3>
                <ResponsiveContainer width="100%" height={300}>
                  <PieChart>
                    <Pie
                      data={typeDistributionData}
                      cx="50%"
                      cy="50%"
                      outerRadius={100}
                      dataKey="count"
                      label={({ type, count }) => `${type} (${count})`}
                    >
                      {typeDistributionData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={entry.color} />
                      ))}
                    </Pie>
                    <Tooltip />
                  </PieChart>
                </ResponsiveContainer>
              </ChartContainer>

              <ChartContainer>
                <h3 style={{ color: '#ffffff', marginBottom: '20px' }}>Keeper Performance</h3>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={keeperStatsData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#333" />
                    <XAxis dataKey="name" stroke="#ffffff" />
                    <YAxis stroke="#ffffff" />
                    <Tooltip 
                      contentStyle={{ 
                        backgroundColor: '#1a1a2e', 
                        border: '1px solid #4facfe',
                        borderRadius: '8px'
                      }}
                    />
                    <Bar dataKey="reputation" fill="#4facfe" name="Reputation" />
                    <Bar dataKey="snts" fill="#00f2fe" name="SNT Count" />
                  </BarChart>
                </ResponsiveContainer>
              </ChartContainer>
            </motion.div>
          )}
        </AnimatePresence>
      </ContentArea>
    </DashboardContainer>
  );
};