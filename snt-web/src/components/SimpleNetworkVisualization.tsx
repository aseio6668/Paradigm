import React from 'react';
import { motion } from 'framer-motion';
import styled from 'styled-components';
import { Keeper, SNT } from '../types/snt';
import { getElementColor, getKeeperStatusSymbol } from '../utils/sntHelpers';

interface SimpleNetworkVisualizationProps {
  keepers: Keeper[];
  snts: SNT[];
}

const VisualizationContainer = styled.div`
  width: 100%;
  height: 400px;
  border-radius: 16px;
  background: radial-gradient(circle at center, #1a1a2e 0%, #16213e 50%, #0f0f23 100%);
  border: 2px solid rgba(100, 200, 255, 0.3);
  position: relative;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
`;

const NetworkGrid = styled.div`
  display: flex;
  gap: 60px;
  align-items: center;
  flex-wrap: wrap;
  justify-content: center;
  padding: 20px;
`;

const KeeperNode = styled(motion.div)<{ $color: string }>`
  display: flex;
  flex-direction: column;
  align-items: center;
  position: relative;
`;

const NodeCircle = styled(motion.div)<{ $color: string; $size: number }>`
  width: ${props => props.$size}px;
  height: ${props => props.$size}px;
  border-radius: 50%;
  background: radial-gradient(circle at 30% 30%, ${props => props.$color}aa, ${props => props.$color}44);
  border: 3px solid ${props => props.$color};
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  box-shadow: 0 0 20px ${props => props.$color}66;
  position: relative;
`;

const KeeperInfo = styled.div`
  margin-top: 12px;
  text-align: center;
  color: #ffffff;
  font-size: 14px;
`;

const SNTOrbit = styled(motion.div)<{ $color: string; $delay: number }>`
  position: absolute;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: ${props => props.$color};
  box-shadow: 0 0 8px ${props => props.$color};
`;


export const SimpleNetworkVisualization: React.FC<SimpleNetworkVisualizationProps> = ({ 
  keepers, 
  snts 
}) => {
  const getKeeperColor = (keeper: Keeper): string => {
    const keeperSNTs = snts.filter(snt => snt.holder === keeper.keeper_id);
    if (keeperSNTs.length > 0) {
      return getElementColor(keeperSNTs[0].glyph.element);
    }
    return '#4488ff';
  };

  const getKeeperSize = (keeper: Keeper): number => {
    const baseSize = 80;
    const reputationBonus = keeper.reputation * 20;
    const sntCount = snts.filter(snt => snt.holder === keeper.keeper_id).length;
    return baseSize + reputationBonus + (sntCount * 5);
  };

  return (
    <VisualizationContainer>
      <NetworkGrid>
        {keepers.map((keeper, index) => {
          const keeperSNTs = snts.filter(snt => snt.holder === keeper.keeper_id);
          const color = getKeeperColor(keeper);
          const size = getKeeperSize(keeper);
          
          return (
            <KeeperNode
              key={keeper.keeper_id}
              $color={color}
              initial={{ opacity: 0, scale: 0 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ duration: 0.5, delay: index * 0.2 }}
            >
              <NodeCircle
                $color={color}
                $size={size}
                whileHover={{ scale: 1.1 }}
                animate={{
                  boxShadow: [
                    `0 0 20px ${color}66`,
                    `0 0 30px ${color}aa`,
                    `0 0 20px ${color}66`
                  ]
                }}
                transition={{
                  boxShadow: {
                    duration: 2,
                    repeat: Infinity,
                    ease: "easeInOut"
                  }
                }}
              >
                {getKeeperStatusSymbol(keeper.status)}
                
                {/* SNT Orbits */}
                {keeperSNTs.slice(0, 3).map((snt, sntIndex) => (
                  <SNTOrbit
                    key={snt.snt_id}
                    $color={getElementColor(snt.glyph.element)}
                    $delay={sntIndex}
                    animate={{
                      rotate: 360,
                      x: [
                        Math.cos((sntIndex * 120) * Math.PI / 180) * (size/2 + 15),
                        Math.cos((sntIndex * 120 + 60) * Math.PI / 180) * (size/2 + 15),
                        Math.cos((sntIndex * 120 + 120) * Math.PI / 180) * (size/2 + 15),
                        Math.cos((sntIndex * 120 + 180) * Math.PI / 180) * (size/2 + 15),
                        Math.cos((sntIndex * 120 + 240) * Math.PI / 180) * (size/2 + 15),
                        Math.cos((sntIndex * 120 + 300) * Math.PI / 180) * (size/2 + 15),
                        Math.cos((sntIndex * 120) * Math.PI / 180) * (size/2 + 15)
                      ],
                      y: [
                        Math.sin((sntIndex * 120) * Math.PI / 180) * (size/2 + 15),
                        Math.sin((sntIndex * 120 + 60) * Math.PI / 180) * (size/2 + 15),
                        Math.sin((sntIndex * 120 + 120) * Math.PI / 180) * (size/2 + 15),
                        Math.sin((sntIndex * 120 + 180) * Math.PI / 180) * (size/2 + 15),
                        Math.sin((sntIndex * 120 + 240) * Math.PI / 180) * (size/2 + 15),
                        Math.sin((sntIndex * 120 + 300) * Math.PI / 180) * (size/2 + 15),
                        Math.sin((sntIndex * 120) * Math.PI / 180) * (size/2 + 15)
                      ]
                    }}
                    transition={{
                      duration: 8 + sntIndex * 2,
                      repeat: Infinity,
                      ease: "linear"
                    }}
                  />
                ))}
              </NodeCircle>
              
              <KeeperInfo>
                <div style={{ fontWeight: 'bold' }}>{keeper.name}</div>
                <div style={{ fontSize: '12px', opacity: 0.8 }}>
                  {keeperSNTs.length} SNTs • {(keeper.reputation * 100).toFixed(0)}% Rep
                </div>
              </KeeperInfo>
            </KeeperNode>
          );
        })}
      </NetworkGrid>
      
      {/* Network Status */}
      <div style={{
        position: 'absolute',
        top: '16px',
        left: '16px',
        background: 'rgba(0, 0, 0, 0.7)',
        padding: '12px',
        borderRadius: '8px',
        fontSize: '14px'
      }}>
        <div style={{ color: '#4facfe', fontWeight: 'bold' }}>🌐 Live Network</div>
        <div style={{ color: '#ffffff', marginTop: '4px' }}>
          {keepers.length} Keepers • {snts.length} SNTs
        </div>
      </div>
    </VisualizationContainer>
  );
};