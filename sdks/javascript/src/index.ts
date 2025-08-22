/**
 * Paradigm JavaScript/TypeScript SDK
 * Official SDK for interacting with the Paradigm blockchain network
 */

export { ParadigmClient } from './client';
export { ParadigmWebSocket } from './websocket';
export { ParadigmWallet } from './wallet';

// Types
export * from './types';

// Utilities
export * from './utils';

// Constants
export { NETWORKS, API_VERSION } from './constants';

// Error classes
export { ParadigmError, NetworkError, ValidationError } from './errors';

// Re-export commonly used interfaces
export type {
  Transaction,
  Block,
  Account,
  MLTask,
  Proposal,
  CrossChainTransfer,
  WebSocketMessage,
  SubscriptionRequest,
} from './types';