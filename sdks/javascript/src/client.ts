import axios, { AxiosInstance, AxiosResponse } from 'axios';
import { 
  ApiResponse, 
  PaginatedResponse, 
  Transaction, 
  Block, 
  Account, 
  CreateTransactionRequest, 
  MLTaskRequest,
  MLTask,
  Proposal,
  CreateProposalRequest,
  NetworkStats,
  TransactionReceipt,
  FeeEstimate
} from './types';
import { ParadigmError, NetworkError, ValidationError } from './errors';
import { validateAddress, validateAmount, formatAddress } from './utils';
import { API_VERSION, NETWORKS } from './constants';

export interface ClientConfig {
  baseURL: string;
  apiKey?: string;
  timeout?: number;
  retries?: number;
  network?: keyof typeof NETWORKS;
}

export class ParadigmClient {
  private http: AxiosInstance;
  private config: ClientConfig;

  constructor(config: ClientConfig) {
    this.config = {
      timeout: 30000,
      retries: 3,
      network: 'mainnet',
      ...config,
    };

    this.http = axios.create({
      baseURL: `${this.config.baseURL}/api/${API_VERSION}`,
      timeout: this.config.timeout,
      headers: {
        'Content-Type': 'application/json',
        'User-Agent': `Paradigm-SDK-JS/1.0.0`,
        ...(this.config.apiKey && { 'X-API-Key': this.config.apiKey }),
      },
    });

    this.setupInterceptors();
  }

  private setupInterceptors(): void {
    // Request interceptor
    this.http.interceptors.request.use(
      (config) => {
        console.debug(`[Paradigm SDK] ${config.method?.toUpperCase()} ${config.url}`);
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Response interceptor
    this.http.interceptors.response.use(
      (response) => response,
      async (error) => {
        const { config, response } = error;
        
        // Retry logic
        if (config.retryCount < this.config.retries! && this.shouldRetry(error)) {
          config.retryCount = (config.retryCount || 0) + 1;
          console.debug(`[Paradigm SDK] Retrying request (${config.retryCount}/${this.config.retries})`);
          await this.delay(1000 * config.retryCount);
          return this.http(config);
        }

        // Convert to custom error
        if (response?.data?.error) {
          throw new ParadigmError(response.data.error.message, response.data.error.code);
        }
        
        throw new NetworkError(`Request failed: ${error.message}`);
      }
    );
  }

  private shouldRetry(error: any): boolean {
    return (
      !error.response ||
      error.response.status >= 500 ||
      error.response.status === 429 ||
      error.code === 'ECONNRESET' ||
      error.code === 'ETIMEDOUT'
    );
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  // Health and status methods
  async getHealth(): Promise<any> {
    const response = await this.http.get('/health');
    return response.data;
  }

  async getNetworkStats(): Promise<NetworkStats> {
    const response = await this.http.get<ApiResponse<NetworkStats>>('/analytics/network-stats');
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to get network stats');
    }
    return response.data.data!;
  }

  // Account methods
  async getAccount(address: string): Promise<Account> {
    if (!validateAddress(address)) {
      throw new ValidationError('Invalid address format');
    }

    const response = await this.http.get<ApiResponse<Account>>(`/accounts/${address}`);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to get account');
    }
    return response.data.data!;
  }

  async getBalance(address: string): Promise<{ balance: string; pending: string; locked: string }> {
    if (!validateAddress(address)) {
      throw new ValidationError('Invalid address format');
    }

    const response = await this.http.get<ApiResponse<any>>(`/accounts/${address}/balance`);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to get balance');
    }
    return response.data.data!;
  }

  // Transaction methods
  async getTransaction(hash: string): Promise<Transaction> {
    const response = await this.http.get<ApiResponse<Transaction>>(`/transactions/${hash}`);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Transaction not found');
    }
    return response.data.data!;
  }

  async getTransactionReceipt(hash: string): Promise<TransactionReceipt> {
    const response = await this.http.get<ApiResponse<TransactionReceipt>>(`/transactions/${hash}/receipt`);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Transaction receipt not found');
    }
    return response.data.data!;
  }

  async createTransaction(request: CreateTransactionRequest): Promise<Transaction> {
    // Validate request
    if (!validateAddress(formatAddress(request.to))) {
      throw new ValidationError('Invalid recipient address');
    }
    if (!validateAmount(request.amount)) {
      throw new ValidationError('Invalid amount');
    }

    const response = await this.http.post<ApiResponse<Transaction>>('/transactions', request);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to create transaction');
    }
    return response.data.data!;
  }

  async sendSignedTransaction(signedTransaction: string): Promise<Transaction> {
    const response = await this.http.post<ApiResponse<Transaction>>('/transactions/send', {
      signed_transaction: signedTransaction,
    });
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to send transaction');
    }
    return response.data.data!;
  }

  async estimateFee(request: CreateTransactionRequest): Promise<FeeEstimate> {
    const response = await this.http.post<ApiResponse<FeeEstimate>>('/transactions/estimate-fee', request);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to estimate fee');
    }
    return response.data.data!;
  }

  async getTransactions(params?: {
    page?: number;
    pageSize?: number;
    address?: string;
  }): Promise<PaginatedResponse<Transaction>> {
    const queryParams = new URLSearchParams();
    if (params?.page) queryParams.append('page', params.page.toString());
    if (params?.pageSize) queryParams.append('page_size', params.pageSize.toString());

    const url = params?.address 
      ? `/addresses/${params.address}/transactions`
      : '/transactions';

    const response = await this.http.get<ApiResponse<PaginatedResponse<Transaction>>>(`${url}?${queryParams}`);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to get transactions');
    }
    return response.data.data!;
  }

  // Block methods
  async getLatestBlock(): Promise<Block> {
    const response = await this.http.get<ApiResponse<Block>>('/blockchain/latest-block');
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to get latest block');
    }
    return response.data.data!;
  }

  async getBlock(height: number): Promise<Block> {
    const response = await this.http.get<ApiResponse<Block>>(`/blockchain/blocks/${height}`);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Block not found');
    }
    return response.data.data!;
  }

  // ML Task methods
  async createMLTask(request: MLTaskRequest): Promise<MLTask> {
    const response = await this.http.post<ApiResponse<MLTask>>('/ml-tasks', request);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to create ML task');
    }
    return response.data.data!;
  }

  async getMLTask(taskId: string): Promise<MLTask> {
    const response = await this.http.get<ApiResponse<MLTask>>(`/ml-tasks/${taskId}`);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'ML task not found');
    }
    return response.data.data!;
  }

  async getMLTasks(params?: {
    page?: number;
    pageSize?: number;
    status?: string;
  }): Promise<PaginatedResponse<MLTask>> {
    const queryParams = new URLSearchParams();
    if (params?.page) queryParams.append('page', params.page.toString());
    if (params?.pageSize) queryParams.append('page_size', params.pageSize.toString());
    if (params?.status) queryParams.append('status', params.status);

    const response = await this.http.get<ApiResponse<PaginatedResponse<MLTask>>>(`/ml-tasks?${queryParams}`);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to get ML tasks');
    }
    return response.data.data!;
  }

  // Governance methods
  async createProposal(request: CreateProposalRequest): Promise<Proposal> {
    const response = await this.http.post<ApiResponse<Proposal>>('/governance/proposals', request);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to create proposal');
    }
    return response.data.data!;
  }

  async getProposal(proposalId: string): Promise<Proposal> {
    const response = await this.http.get<ApiResponse<Proposal>>(`/governance/proposals/${proposalId}`);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Proposal not found');
    }
    return response.data.data!;
  }

  async getProposals(params?: {
    page?: number;
    pageSize?: number;
    status?: string;
  }): Promise<PaginatedResponse<Proposal>> {
    const queryParams = new URLSearchParams();
    if (params?.page) queryParams.append('page', params.page.toString());
    if (params?.pageSize) queryParams.append('page_size', params.pageSize.toString());
    if (params?.status) queryParams.append('status', params.status);

    const response = await this.http.get<ApiResponse<PaginatedResponse<Proposal>>>(`/governance/proposals?${queryParams}`);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to get proposals');
    }
    return response.data.data!;
  }

  async vote(proposalId: string, option: 'yes' | 'no' | 'abstain'): Promise<void> {
    const response = await this.http.post<ApiResponse<void>>(`/governance/proposals/${proposalId}/vote`, {
      option,
    });
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to vote');
    }
  }

  // Cross-chain methods
  async createCrossChainTransfer(request: {
    fromChain: string;
    toChain: string;
    asset: string;
    amount: string;
    recipient: string;
    memo?: string;
  }): Promise<any> {
    const response = await this.http.post<ApiResponse<any>>('/cross-chain/transfer', request);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Failed to create cross-chain transfer');
    }
    return response.data.data!;
  }

  async getCrossChainTransfer(transferId: string): Promise<any> {
    const response = await this.http.get<ApiResponse<any>>(`/cross-chain/transfers/${transferId}`);
    if (!response.data.success) {
      throw new ParadigmError(response.data.error?.message || 'Cross-chain transfer not found');
    }
    return response.data.data!;
  }

  // Utility methods
  setApiKey(apiKey: string): void {
    this.config.apiKey = apiKey;
    this.http.defaults.headers['X-API-Key'] = apiKey;
  }

  setTimeout(timeout: number): void {
    this.config.timeout = timeout;
    this.http.defaults.timeout = timeout;
  }

  getConfig(): Readonly<ClientConfig> {
    return { ...this.config };
  }
}