export interface SNT {
  snt_id: string;
  holder: string;
  token_type: SNTType;
  glyph: Glyph;
  created_at: number;
  evolution_level: number;
  evolution_progress: number;
  permissions: Permission[];
  properties: Record<string, string>;
  narrative_fragments: string[];
}

export enum SNTType {
  KeeperIdentity = "KeeperIdentity",
  StorageContribution = "StorageContribution", 
  MemoryAnchor = "MemoryAnchor",
  FusionMaster = "FusionMaster",
  GlyphArtist = "GlyphArtist",
  CommunityBond = "CommunityBond"
}

export interface Glyph {
  element: Element;
  category: DataCategory;
  importance: Importance;
  symbol: string;
}

export enum Element {
  Fire = "Fire",
  Water = "Water", 
  Earth = "Earth",
  Air = "Air",
  Lightning = "Lightning",
  Void = "Void",
  Aether = "Aether"
}

export enum DataCategory {
  Archive = "Archive",
  Model = "Model",
  Dataset = "Dataset", 
  Result = "Result",
  Media = "Media",
  Document = "Document",
  Code = "Code",
  Identity = "Identity"
}

export enum Importance {
  Trivial = "Trivial",
  Minor = "Minor",
  Standard = "Standard",
  Major = "Major", 
  Critical = "Critical",
  Legendary = "Legendary"
}

export interface Permission {
  resource_type: string;
  access_level: AccessLevel;
  conditions: string[];
}

export enum AccessLevel {
  Read = "Read",
  Write = "Write",
  Execute = "Execute", 
  Admin = "Admin"
}

export interface Keeper {
  keeper_id: string;
  name: string;
  capacity: number;
  used_storage: number;
  reputation: number;
  sigils_hosted: string[];
  total_earned: number;
  status: KeeperStatus;
  created_at: number;
}

export enum KeeperStatus {
  Apprentice = "Apprentice",
  Guardian = "Guardian", 
  Archivist = "Archivist",
  Loremaster = "Loremaster"
}

export interface Sigil {
  sigil_id: string;
  filename?: string;
  size: number;
  glyph: Glyph;
  owner: string;
  stored_at: string[];
  created_at: number;
}

export interface NetworkStats {
  total_snts: number;
  type_distribution: Record<string, number>;
  average_evolution_level: number;
  unique_holders: number;
  active_keepers: number;
  total_sigils: number;
  total_storage_used: number;
  network_utilization: number;
  recent_events: number;
}

export interface NetworkEvent {
  event_type: string;
  actor: string;
  details: string;
  timestamp: number;
}