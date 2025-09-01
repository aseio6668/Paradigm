import { SNT, SNTType, Element, DataCategory, Importance, Keeper, KeeperStatus, Sigil, NetworkEvent } from '../types/snt';

export const mockKeepers: Keeper[] = [
  {
    keeper_id: 'keeper_ac9f8962',
    name: 'Alice',
    capacity: 100 * 1024 * 1024 * 1024,
    used_storage: 55 * 1024 * 1024,
    reputation: 0.95,
    sigils_hosted: ['sigil_abc123', 'sigil_def456'],
    total_earned: 250,
    status: KeeperStatus.Guardian,
    created_at: Date.now() - 86400000
  },
  {
    keeper_id: 'keeper_bc2f7834',
    name: 'Bob',
    capacity: 50 * 1024 * 1024 * 1024,
    used_storage: 20 * 1024 * 1024,
    reputation: 0.87,
    sigils_hosted: ['sigil_ghi789'],
    total_earned: 120,
    status: KeeperStatus.Apprentice,
    created_at: Date.now() - 43200000
  },
  {
    keeper_id: 'keeper_cd3e6745',
    name: 'Carol',
    capacity: 200 * 1024 * 1024 * 1024,
    used_storage: 150 * 1024 * 1024,
    reputation: 0.92,
    sigils_hosted: ['sigil_jkl012', 'sigil_mno345', 'sigil_pqr678'],
    total_earned: 380,
    status: KeeperStatus.Archivist,
    created_at: Date.now() - 172800000
  }
];

export const mockSNTs: SNT[] = [
  {
    snt_id: 'snt_identity_alice',
    holder: 'keeper_ac9f8962',
    token_type: SNTType.KeeperIdentity,
    glyph: {
      element: Element.Earth,
      category: DataCategory.Identity,
      importance: Importance.Major,
      symbol: '🛡️'
    },
    created_at: Date.now() - 86400000,
    evolution_level: 3,
    evolution_progress: 45.0,
    permissions: [],
    properties: {
      'keeper_name': 'Alice',
      'initial_capacity': '100 GB',
      'welcome_bonus': '10_PAR'
    },
    narrative_fragments: [
      "Born from the network's recognition of Alice's contribution",
      "Evolved to level 2 through dedicated network participation",
      "Evolved to level 3 through dedicated network participation"
    ]
  },
  {
    snt_id: 'snt_storage_alice_1',
    holder: 'keeper_ac9f8962',
    token_type: SNTType.StorageContribution,
    glyph: {
      element: Element.Fire,
      category: DataCategory.Archive,
      importance: Importance.Standard,
      symbol: '📦'
    },
    created_at: Date.now() - 82800000,
    evolution_level: 2,
    evolution_progress: 75.0,
    permissions: [],
    properties: {
      'sigil_id': 'sigil_abc123',
      'filename': 'research-data.json',
      'size': '5.0 MB',
      'file_type': '📊 Dataset'
    },
    narrative_fragments: [
      "Born from the network's recognition of Alice's contribution",
      "Evolved to level 2 through dedicated network participation"
    ]
  },
  {
    snt_id: 'snt_memory_alice',
    holder: 'keeper_ac9f8962',
    token_type: SNTType.MemoryAnchor,
    glyph: {
      element: Element.Void,
      category: DataCategory.Archive,
      importance: Importance.Legendary,
      symbol: '📜'
    },
    created_at: Date.now() - 72000000,
    evolution_level: 1,
    evolution_progress: 30.0,
    permissions: [],
    properties: {
      'tome_id': 'tome_synthesis',
      'ritual_type': '⚗️ Synthesis',
      'sigil_count': '2',
      'fusion_quality': 'Perfect'
    },
    narrative_fragments: [
      "Born from the network's recognition of Alice's contribution"
    ]
  },
  {
    snt_id: 'snt_identity_bob',
    holder: 'keeper_bc2f7834',
    token_type: SNTType.KeeperIdentity,
    glyph: {
      element: Element.Earth,
      category: DataCategory.Identity,
      importance: Importance.Major,
      symbol: '🛡️'
    },
    created_at: Date.now() - 43200000,
    evolution_level: 1,
    evolution_progress: 60.0,
    permissions: [],
    properties: {
      'keeper_name': 'Bob',
      'initial_capacity': '50 GB',
      'welcome_bonus': '10_PAR'
    },
    narrative_fragments: [
      "Born from the network's recognition of Bob's contribution"
    ]
  },
  {
    snt_id: 'snt_fusion_carol',
    holder: 'keeper_cd3e6745',
    token_type: SNTType.FusionMaster,
    glyph: {
      element: Element.Aether,
      category: DataCategory.Result,
      importance: Importance.Critical,
      symbol: '⚗️'
    },
    created_at: Date.now() - 86400000,
    evolution_level: 4,
    evolution_progress: 20.0,
    permissions: [],
    properties: {
      'achievement': 'fusion_master',
      'fusion_count': '5'
    },
    narrative_fragments: [
      "Born from the network's recognition of Carol's contribution",
      "Evolved to level 2 through dedicated network participation",
      "Evolved to level 3 through dedicated network participation", 
      "Evolved to level 4 through dedicated network participation"
    ]
  }
];

export const mockSigils: Sigil[] = [
  {
    sigil_id: 'sigil_abc123',
    filename: 'research-data.json',
    size: 5 * 1024 * 1024,
    glyph: {
      element: Element.Fire,
      category: DataCategory.Dataset,
      importance: Importance.Major,
      symbol: '📊'
    },
    owner: 'keeper_ac9f8962',
    stored_at: ['keeper_ac9f8962', 'keeper_cd3e6745'],
    created_at: Date.now() - 82800000
  },
  {
    sigil_id: 'sigil_def456',
    filename: 'ml-model.pkl',
    size: 50 * 1024 * 1024,
    glyph: {
      element: Element.Aether,
      category: DataCategory.Model,
      importance: Importance.Critical,
      symbol: '🧠'
    },
    owner: 'keeper_ac9f8962',
    stored_at: ['keeper_ac9f8962'],
    created_at: Date.now() - 79200000
  },
  {
    sigil_id: 'sigil_ghi789',
    filename: 'image-gallery.zip',
    size: 100 * 1024 * 1024,
    glyph: {
      element: Element.Water,
      category: DataCategory.Media,
      importance: Importance.Major,
      symbol: '🖼️'
    },
    owner: 'keeper_bc2f7834',
    stored_at: ['keeper_bc2f7834', 'keeper_cd3e6745'],
    created_at: Date.now() - 39600000
  }
];

export const mockEvents: NetworkEvent[] = [
  {
    event_type: 'keeper_registered',
    actor: 'keeper_ac9f8962',
    details: 'Alice joined as 🔰 Apprentice keeper',
    timestamp: Date.now() - 86400000
  },
  {
    event_type: 'sigil_stored',
    actor: 'keeper_ac9f8962', 
    details: 'Stored research-data.json (5.0 MB)',
    timestamp: Date.now() - 82800000
  },
  {
    event_type: 'fusion_completed',
    actor: 'keeper_ac9f8962',
    details: 'Performed ⚗️ Synthesis ritual with 2 sigils',
    timestamp: Date.now() - 72000000
  },
  {
    event_type: 'snt_evolved',
    actor: 'keeper_ac9f8962',
    details: 'SNT evolved to next level!',
    timestamp: Date.now() - 64800000
  },
  {
    event_type: 'keeper_registered',
    actor: 'keeper_bc2f7834',
    details: 'Bob joined as 🔰 Apprentice keeper',
    timestamp: Date.now() - 43200000
  }
];