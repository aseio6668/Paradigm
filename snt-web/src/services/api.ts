import { SNT, Keeper, NetworkStats, NetworkEvent } from '../types/snt';

const API_BASE_URL = 'http://localhost:8080/api';

export class ParadigmAPI {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private async fetchJSON(endpoint: string, options: RequestInit = {}): Promise<any> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
        ...options,
      });

      if (!response.ok) {
        throw new Error(`API Error: ${response.status} ${response.statusText}`);
      }

      const data = await response.json();
      return data.success ? data.data : data;
    } catch (error) {
      console.error(`API request failed for ${endpoint}:`, error);
      // Return mock data if API is not available
      return this.getMockData(endpoint);
    }
  }

  private getMockData(endpoint: string): any {
    // Return appropriate mock data based on endpoint
    if (endpoint.includes('/network/stats')) {
      return {
        total_snts: 5,
        active_keepers: 3,
        unique_holders: 2,
        total_sigils: 3,
        average_evolution_level: 2.2,
        total_storage_used: 175 * 1024 * 1024,
        network_utilization: 15.5,
        recent_events: 8
      };
    }
    
    return [];
  }

  // Network APIs
  async getNetworkStats(): Promise<NetworkStats> {
    return this.fetchJSON('/network/stats');
  }

  async getNetworkEvents(): Promise<NetworkEvent[]> {
    return this.fetchJSON('/network/events');
  }

  // Keeper APIs
  async getKeepers(): Promise<Keeper[]> {
    return this.fetchJSON('/keepers/list');
  }

  // SNT APIs
  async getSNTs(): Promise<SNT[]> {
    return this.fetchJSON('/snt/list');
  }

  // Health check
  async checkHealth(): Promise<{ status: string; timestamp: number }> {
    try {
      const response = await fetch(`${this.baseUrl.replace('/api', '')}/health`);
      if (response.ok) {
        return { status: 'online', timestamp: Date.now() };
      }
    } catch (error) {
      // Network is not running, use mock data
    }
    return { status: 'mock', timestamp: Date.now() };
  }
}

// Export singleton instance
export const paradigmAPI = new ParadigmAPI();