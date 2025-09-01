import React, { useRef, useMemo } from 'react';
import { Canvas, useFrame, useThree } from '@react-three/fiber';
import { Sphere, Line, Text } from '@react-three/drei';
import * as THREE from 'three';
import { motion } from 'framer-motion';
import styled from 'styled-components';
import { Keeper, SNT } from '../types/snt';
import { getElementColor, getKeeperStatusSymbol } from '../utils/sntHelpers';

interface NetworkVisualizationProps {
  keepers: Keeper[];
  snts: SNT[];
}

const VisualizationContainer = styled(motion.div)`
  width: 100%;
  height: 400px;
  border-radius: 16px;
  overflow: hidden;
  background: radial-gradient(circle at center, #1a1a2e 0%, #16213e 50%, #0f0f23 100%);
  border: 2px solid rgba(100, 200, 255, 0.3);
  position: relative;
`;

const Legend = styled.div`
  position: absolute;
  top: 16px;
  right: 16px;
  background: rgba(0, 0, 0, 0.7);
  border-radius: 8px;
  padding: 12px;
  z-index: 10;
  font-size: 12px;
`;

const LegendItem = styled.div`
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
  color: #ffffff;
`;

const ColorDot = styled.div<{ $color: string }>`
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: ${props => props.$color};
  box-shadow: 0 0 8px ${props => props.$color};
`;

// Keeper Node Component
const KeeperNode: React.FC<{ 
  keeper: Keeper; 
  position: [number, number, number]; 
  snts: SNT[];
}> = ({ keeper, position, snts }) => {
  const meshRef = useRef<THREE.Mesh>(null);
  const keeperSNTs = snts.filter(snt => snt.holder === keeper.keeper_id);
  
  // Calculate node size based on reputation and SNT count
  const nodeSize = 0.3 + (keeper.reputation * 0.5) + (keeperSNTs.length * 0.1);
  
  // Calculate primary color from most common element
  const elementColors = keeperSNTs.map(snt => getElementColor(snt.glyph.element));
  const primaryColor = elementColors.length > 0 ? elementColors[0] : '#4488ff';

  useFrame((state) => {
    if (meshRef.current) {
      // Gentle pulsing animation
      meshRef.current.scale.setScalar(
        nodeSize + Math.sin(state.clock.elapsedTime * 2 + position[0]) * 0.1
      );
    }
  });

  return (
    <group position={position}>
      <Sphere ref={meshRef} args={[nodeSize, 16, 16]}>
        <meshPhongMaterial 
          color={primaryColor}
          transparent
          opacity={0.8}
          emissive={primaryColor}
          emissiveIntensity={0.2}
        />
      </Sphere>
      
      {/* Keeper name */}
      <Text
        position={[0, nodeSize + 0.4, 0]}
        fontSize={0.3}
        color="#ffffff"
        anchorX="center"
        anchorY="middle"
      >
        {getKeeperStatusSymbol(keeper.status)} {keeper.name}
      </Text>
      
      {/* SNT orbits */}
      {keeperSNTs.map((snt, index) => (
        <SNTOrbit 
          key={snt.snt_id}
          snt={snt}
          radius={nodeSize + 0.5 + (index * 0.3)}
          speed={1 + index * 0.3}
        />
      ))}
    </group>
  );
};

// SNT Orbit Component
const SNTOrbit: React.FC<{
  snt: SNT;
  radius: number;
  speed: number;
}> = ({ snt, radius, speed }) => {
  const meshRef = useRef<THREE.Mesh>(null);
  const color = getElementColor(snt.glyph.element);
  
  useFrame((state) => {
    if (meshRef.current) {
      const angle = state.clock.elapsedTime * speed;
      meshRef.current.position.x = Math.cos(angle) * radius;
      meshRef.current.position.z = Math.sin(angle) * radius;
      
      // Slight vertical oscillation
      meshRef.current.position.y = Math.sin(angle * 2) * 0.1;
    }
  });

  return (
    <Sphere ref={meshRef} args={[0.05, 8, 8]}>
      <meshPhongMaterial 
        color={color}
        emissive={color}
        emissiveIntensity={0.5}
      />
    </Sphere>
  );
};

// Connection Lines Component
const ConnectionLines: React.FC<{ keepers: Keeper[]; positions: [number, number, number][] }> = ({ 
  keepers, 
  positions 
}) => {
  const connections = useMemo(() => {
    const lines: Array<{
      from: [number, number, number];
      to: [number, number, number];
      strength: number;
    }> = [];
    
    for (let i = 0; i < keepers.length; i++) {
      for (let j = i + 1; j < keepers.length; j++) {
        // Create connections based on shared sigils or similar reputation
        const keeper1 = keepers[i];
        const keeper2 = keepers[j];
        
        const sharedSigils = keeper1.sigils_hosted.filter(sigil => 
          keeper2.sigils_hosted.includes(sigil)
        ).length;
        
        const reputationSimilarity = 1 - Math.abs(keeper1.reputation - keeper2.reputation);
        
        if (sharedSigils > 0 || reputationSimilarity > 0.8) {
          lines.push({
            from: positions[i],
            to: positions[j],
            strength: sharedSigils > 0 ? sharedSigils * 0.5 : reputationSimilarity * 0.3
          });
        }
      }
    }
    
    return lines;
  }, [keepers, positions]);

  return (
    <>
      {connections.map((connection, index) => (
        <Line
          key={index}
          points={[connection.from, connection.to]}
          color="#4488ff"
          lineWidth={connection.strength * 2}
          transparent
          opacity={connection.strength}
        />
      ))}
    </>
  );
};

// Main Network Scene
const NetworkScene: React.FC<{ keepers: Keeper[]; snts: SNT[] }> = ({ keepers, snts }) => {
  // Generate positions for keepers in a circle
  const positions = useMemo(() => {
    return keepers.map((_, index) => {
      const angle = (index / keepers.length) * Math.PI * 2;
      const radius = 3;
      return [
        Math.cos(angle) * radius,
        0,
        Math.sin(angle) * radius
      ] as [number, number, number];
    });
  }, [keepers]);

  return (
    <>
      <ambientLight intensity={0.3} />
      <pointLight position={[10, 10, 10]} intensity={0.8} />
      <pointLight position={[-10, -10, -10]} intensity={0.4} color="#ff4488" />
      
      {/* Keeper nodes */}
      {keepers.map((keeper, index) => (
        <KeeperNode
          key={keeper.keeper_id}
          keeper={keeper}
          position={positions[index]}
          snts={snts}
        />
      ))}
      
      {/* Connection lines */}
      <ConnectionLines keepers={keepers} positions={positions} />
      
      {/* Background particles */}
      <Stars />
    </>
  );
};

// Background Stars
const Stars: React.FC = () => {
  const starsRef = useRef<THREE.Points>(null);
  
  const starGeometry = useMemo(() => {
    const geometry = new THREE.BufferGeometry();
    const vertices = [];
    
    for (let i = 0; i < 1000; i++) {
      vertices.push(
        (Math.random() - 0.5) * 20,
        (Math.random() - 0.5) * 20,
        (Math.random() - 0.5) * 20
      );
    }
    
    geometry.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
    return geometry;
  }, []);

  useFrame(() => {
    if (starsRef.current) {
      starsRef.current.rotation.y += 0.001;
      starsRef.current.rotation.x += 0.0005;
    }
  });

  return (
    <points ref={starsRef} geometry={starGeometry}>
      <pointsMaterial color="#ffffff" size={0.02} transparent opacity={0.6} />
    </points>
  );
};

export const NetworkVisualization: React.FC<NetworkVisualizationProps> = ({ keepers, snts }) => {
  const elementTypes = Array.from(new Set(snts.map(snt => snt.glyph.element)));
  
  return (
    <VisualizationContainer
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.5 }}
    >
      <Canvas camera={{ position: [8, 4, 8], fov: 60 }}>
        <NetworkScene keepers={keepers} snts={snts} />
      </Canvas>
      
      <Legend>
        <div style={{ fontWeight: 'bold', marginBottom: '8px', color: '#ffffff' }}>
          🌐 Network Legend
        </div>
        {elementTypes.slice(0, 4).map(element => (
          <LegendItem key={element}>
            <ColorDot $color={getElementColor(element)} />
            <span>{element} Element</span>
          </LegendItem>
        ))}
        <LegendItem>
          <ColorDot $color="#4488ff" />
          <span>Keeper Connections</span>
        </LegendItem>
      </Legend>
    </VisualizationContainer>
  );
};