import { Element, Importance, SNTType, KeeperStatus } from '../types/snt';

export const getElementSymbol = (element: Element): string => {
  switch (element) {
    case Element.Fire: return '🔥';
    case Element.Water: return '💧';
    case Element.Earth: return '🌍';
    case Element.Air: return '💨';
    case Element.Lightning: return '⚡';
    case Element.Void: return '🌙';
    case Element.Aether: return '🔮';
    default: return '🌍';
  }
};

export const getElementColor = (element: Element): string => {
  switch (element) {
    case Element.Fire: return '#ff4444';
    case Element.Water: return '#4488ff';
    case Element.Earth: return '#44aa44';
    case Element.Air: return '#cccccc';
    case Element.Lightning: return '#ffff44';
    case Element.Void: return '#8844cc';
    case Element.Aether: return '#ff44cc';
    default: return '#44aa44';
  }
};

export const getSNTTypeName = (type: SNTType): string => {
  switch (type) {
    case SNTType.KeeperIdentity: return '🛡️ Keeper Identity';
    case SNTType.StorageContribution: return '📦 Storage Contribution';
    case SNTType.MemoryAnchor: return '📜 Memory Anchor';
    case SNTType.FusionMaster: return '⚗️ Fusion Master';
    case SNTType.GlyphArtist: return '🎨 Glyph Artist';
    case SNTType.CommunityBond: return '🤝 Community Bond';
  }
};

export const getImportanceColor = (importance: Importance): string => {
  switch (importance) {
    case Importance.Trivial: return '#666666';
    case Importance.Minor: return '#888888';
    case Importance.Standard: return '#aaaaaa';
    case Importance.Major: return '#ffaa00';
    case Importance.Critical: return '#ff6600';
    case Importance.Legendary: return '#ff0066';
    default: return '#aaaaaa';
  }
};

export const getKeeperStatusSymbol = (status: KeeperStatus): string => {
  switch (status) {
    case KeeperStatus.Apprentice: return '🔰';
    case KeeperStatus.Guardian: return '🛡️';
    case KeeperStatus.Archivist: return '📚';
    case KeeperStatus.Loremaster: return '🎭';
  }
};

export const formatFileSize = (bytes: number): string => {
  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  } else if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  } else if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  } else {
    return `${bytes} B`;
  }
};

export const getEvolutionThreshold = (level: number): number => {
  return level * 100 + 50;
};

export const generateId = (): string => {
  return Math.random().toString(36).substr(2, 8);
};