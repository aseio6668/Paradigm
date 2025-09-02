import React from 'react';
import { motion } from 'framer-motion';
import styled from 'styled-components';
import { SNT } from '../types/snt';
import { getElementSymbol, getElementColor, getSNTTypeName, getImportanceColor } from '../utils/sntHelpers';

interface SNTCardProps {
  snt: SNT;
  onClick?: () => void;
  compact?: boolean;
}

const CardContainer = styled(motion.div)<{ $elementColor: string; $importanceColor: string; $compact?: boolean; onClick?: () => void }>`
  background: linear-gradient(135deg, 
    rgba(${props => hexToRgb(props.$elementColor)}, 0.2) 0%,
    rgba(${props => hexToRgb(props.$importanceColor)}, 0.1) 100%);
  border: 2px solid rgba(${props => hexToRgb(props.$elementColor)}, 0.5);
  border-radius: 16px;
  padding: ${props => props.$compact ? '16px' : '24px'};
  cursor: pointer;
  position: relative;
  overflow: hidden;
  backdrop-filter: blur(10px);
  min-height: ${props => props.$compact ? '120px' : '200px'};
  
  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 4px;
    background: linear-gradient(90deg, ${props => props.$elementColor}, ${props => props.$importanceColor});
  }
  
  &:hover {
    border-color: ${props => props.$elementColor};
    box-shadow: 0 8px 32px rgba(${props => hexToRgb(props.$elementColor)}, 0.3);
  }
`;

const HeaderRow = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
`;

const SNTTitle = styled.div`
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: bold;
  color: #ffffff;
`;

const ElementSymbol = styled.span<{ $color: string }>`
  font-size: 24px;
  filter: drop-shadow(0 0 8px ${props => props.$color});
`;

const EvolutionLevel = styled.div<{ $color: string }>`
  background: rgba(${props => hexToRgb(props.$color)}, 0.3);
  border: 1px solid rgba(${props => hexToRgb(props.$color)}, 0.5);
  border-radius: 20px;
  padding: 4px 12px;
  font-size: 14px;
  font-weight: bold;
  color: ${props => props.$color};
`;

const ProgressBar = styled.div<{ $color: string }>`
  width: 100%;
  height: 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  overflow: hidden;
  margin: 12px 0;
`;

const ProgressFill = styled(motion.div)<{ $color: string }>`
  height: 100%;
  background: linear-gradient(90deg, ${props => props.$color}, ${props => props.$color}aa);
  border-radius: 4px;
`;

const NarrativeFragment = styled.div`
  font-size: 12px;
  color: #cccccc;
  font-style: italic;
  margin-top: 8px;
  opacity: 0.8;
`;

const Properties = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 8px;
  margin-top: 12px;
  font-size: 12px;
`;

const Property = styled.div`
  color: #aaaaaa;
  
  .key {
    font-weight: bold;
    color: #ffffff;
  }
`;

function hexToRgb(hex: string): string {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) return '255, 255, 255';
  
  return `${parseInt(result[1], 16)}, ${parseInt(result[2], 16)}, ${parseInt(result[3], 16)}`;
}

export const SNTCard: React.FC<SNTCardProps> = ({ snt, onClick, compact = false }) => {
  const elementColor = getElementColor(snt.glyph.element);
  const importanceColor = getImportanceColor(snt.glyph.importance);

  return (
    <CardContainer
      $elementColor={elementColor}
      $importanceColor={importanceColor}
      $compact={compact}
      onClick={onClick}
      whileHover={{ scale: 1.02, y: -4 }}
      whileTap={{ scale: 0.98 }}
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
    >
      <HeaderRow>
        <SNTTitle>
          <ElementSymbol $color={elementColor}>
            {getElementSymbol(snt.glyph.element)}
          </ElementSymbol>
          <span>{getSNTTypeName(snt.token_type)}</span>
        </SNTTitle>
        <EvolutionLevel $color={importanceColor}>
          Level {snt.evolution_level}
        </EvolutionLevel>
      </HeaderRow>

      <ProgressBar $color={elementColor}>
        <ProgressFill
          $color={elementColor}
          initial={{ width: 0 }}
          animate={{ width: `${snt.evolution_progress}%` }}
          transition={{ duration: 1, delay: 0.2 }}
        />
      </ProgressBar>

      <div style={{ fontSize: '14px', color: '#ffffff', marginBottom: '8px' }}>
        Progress: {snt.evolution_progress.toFixed(1)}%
      </div>

      {!compact && (
        <>
          {snt.narrative_fragments.length > 0 && (
            <NarrativeFragment>
              "{snt.narrative_fragments[snt.narrative_fragments.length - 1]}"
            </NarrativeFragment>
          )}

          <Properties>
            {Object.entries(snt.properties).slice(0, 4).map(([key, value]) => (
              <Property key={key}>
                <span className="key">{key.replace('_', ' ')}:</span> {value}
              </Property>
            ))}
          </Properties>
        </>
      )}
    </CardContainer>
  );
};