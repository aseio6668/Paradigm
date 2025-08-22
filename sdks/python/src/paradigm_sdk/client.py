"""
Paradigm API Client

Provides a high-level interface for interacting with the Paradigm blockchain
network via REST API endpoints.
"""

import asyncio
import logging
from typing import Dict, List, Optional, Union, Any
from dataclasses import dataclass
import json
import time

import httpx
from httpx import AsyncClient, Client

from .exceptions import ParadigmError, NetworkError, ValidationError, RateLimitError
from .types import (
    Transaction, Block, Account, MLTask, Proposal, NetworkStats,
    TransactionReceipt, FeeEstimate, PaginatedResponse,
    CreateTransactionRequest, MLTaskRequest, CreateProposalRequest
)
from .utils import validate_address, validate_amount, format_address
from .constants import API_VERSION, NETWORKS

logger = logging.getLogger(__name__)


@dataclass
class ClientConfig:
    """Configuration for ParadigmClient"""
    base_url: str
    api_key: Optional[str] = None
    timeout: float = 30.0
    retries: int = 3
    network: str = "mainnet"
    rate_limit_requests: int = 100
    rate_limit_window: int = 60  # seconds


class ParadigmClient:
    """
    Main client for interacting with Paradigm blockchain network.
    
    Provides methods for:
    - Account management
    - Transaction operations  
    - Block queries
    - ML task management
    - Governance participation
    - Cross-chain operations
    """

    def __init__(self, config: Union[ClientConfig, Dict[str, Any]]):
        """
        Initialize Paradigm client.
        
        Args:
            config: Client configuration (ClientConfig object or dict)
        """
        if isinstance(config, dict):
            self.config = ClientConfig(**config)
        else:
            self.config = config
            
        self._setup_http_client()
        self._rate_limiter = RateLimiter(
            self.config.rate_limit_requests,
            self.config.rate_limit_window
        )

    def _setup_http_client(self) -> None:
        """Setup HTTP client with proper headers and timeouts."""
        headers = {
            "Content-Type": "application/json",
            "User-Agent": f"Paradigm-SDK-Python/1.0.0",
        }
        
        if self.config.api_key:
            headers["X-API-Key"] = self.config.api_key
            
        self._client = Client(
            base_url=f"{self.config.base_url}/api/{API_VERSION}",
            headers=headers,
            timeout=self.config.timeout,
        )
        
        self._async_client = AsyncClient(
            base_url=f"{self.config.base_url}/api/{API_VERSION}",
            headers=headers,
            timeout=self.config.timeout,
        )

    def _handle_response(self, response: httpx.Response) -> Dict[str, Any]:
        """Handle HTTP response and convert errors."""
        try:
            data = response.json()
        except json.JSONDecodeError:
            raise NetworkError(f"Invalid JSON response: {response.text}")
            
        if response.status_code == 429:
            raise RateLimitError("Rate limit exceeded")
        elif response.status_code >= 400:
            error_msg = data.get("error", {}).get("message", "Unknown error")
            error_code = data.get("error", {}).get("code", "UNKNOWN")
            raise ParadigmError(error_msg, error_code)
            
        if not data.get("success", False):
            error_msg = data.get("error", {}).get("message", "Request failed")
            raise ParadigmError(error_msg)
            
        return data.get("data", {})

    def _make_request(
        self, 
        method: str, 
        endpoint: str, 
        params: Optional[Dict] = None,
        json_data: Optional[Dict] = None
    ) -> Dict[str, Any]:
        """Make HTTP request with retry logic."""
        self._rate_limiter.wait_if_needed()
        
        for attempt in range(self.config.retries + 1):
            try:
                response = self._client.request(
                    method=method,
                    url=endpoint,
                    params=params,
                    json=json_data
                )
                return self._handle_response(response)
                
            except (httpx.ConnectError, httpx.TimeoutException) as e:
                if attempt == self.config.retries:
                    raise NetworkError(f"Request failed after {self.config.retries} retries: {e}")
                    
                wait_time = 2 ** attempt
                logger.warning(f"Request failed, retrying in {wait_time}s... (attempt {attempt + 1})")
                time.sleep(wait_time)

    async def _make_async_request(
        self, 
        method: str, 
        endpoint: str, 
        params: Optional[Dict] = None,
        json_data: Optional[Dict] = None
    ) -> Dict[str, Any]:
        """Make async HTTP request with retry logic."""
        await self._rate_limiter.async_wait_if_needed()
        
        for attempt in range(self.config.retries + 1):
            try:
                response = await self._async_client.request(
                    method=method,
                    url=endpoint,
                    params=params,
                    json=json_data
                )
                return self._handle_response(response)
                
            except (httpx.ConnectError, httpx.TimeoutException) as e:
                if attempt == self.config.retries:
                    raise NetworkError(f"Request failed after {self.config.retries} retries: {e}")
                    
                wait_time = 2 ** attempt
                logger.warning(f"Request failed, retrying in {wait_time}s... (attempt {attempt + 1})")
                await asyncio.sleep(wait_time)

    # Health and Status Methods
    
    def get_health(self) -> Dict[str, Any]:
        """Get API health status."""
        return self._make_request("GET", "/health")

    async def get_health_async(self) -> Dict[str, Any]:
        """Get API health status (async)."""
        return await self._make_async_request("GET", "/health")

    def get_network_stats(self) -> NetworkStats:
        """Get network statistics."""
        data = self._make_request("GET", "/analytics/network-stats")
        return NetworkStats(**data)

    async def get_network_stats_async(self) -> NetworkStats:
        """Get network statistics (async)."""
        data = await self._make_async_request("GET", "/analytics/network-stats")
        return NetworkStats(**data)

    # Account Methods
    
    def get_account(self, address: str) -> Account:
        """Get account information."""
        if not validate_address(address):
            raise ValidationError("Invalid address format")
            
        data = self._make_request("GET", f"/accounts/{address}")
        return Account(**data)

    async def get_account_async(self, address: str) -> Account:
        """Get account information (async)."""
        if not validate_address(address):
            raise ValidationError("Invalid address format")
            
        data = await self._make_async_request("GET", f"/accounts/{address}")
        return Account(**data)

    def get_balance(self, address: str) -> Dict[str, str]:
        """Get account balance."""
        if not validate_address(address):
            raise ValidationError("Invalid address format")
            
        return self._make_request("GET", f"/accounts/{address}/balance")

    async def get_balance_async(self, address: str) -> Dict[str, str]:
        """Get account balance (async)."""
        if not validate_address(address):
            raise ValidationError("Invalid address format")
            
        return await self._make_async_request("GET", f"/accounts/{address}/balance")

    # Transaction Methods
    
    def get_transaction(self, hash: str) -> Transaction:
        """Get transaction by hash."""
        data = self._make_request("GET", f"/transactions/{hash}")
        return Transaction(**data)

    async def get_transaction_async(self, hash: str) -> Transaction:
        """Get transaction by hash (async)."""
        data = await self._make_async_request("GET", f"/transactions/{hash}")
        return Transaction(**data)

    def create_transaction(self, request: CreateTransactionRequest) -> Transaction:
        """Create a new transaction."""
        if not validate_address(format_address(request.to)):
            raise ValidationError("Invalid recipient address")
        if not validate_amount(request.amount):
            raise ValidationError("Invalid amount")
            
        data = self._make_request("POST", "/transactions", json_data=request.__dict__)
        return Transaction(**data)

    async def create_transaction_async(self, request: CreateTransactionRequest) -> Transaction:
        """Create a new transaction (async)."""
        if not validate_address(format_address(request.to)):
            raise ValidationError("Invalid recipient address")
        if not validate_amount(request.amount):
            raise ValidationError("Invalid amount")
            
        data = await self._make_async_request("POST", "/transactions", json_data=request.__dict__)
        return Transaction(**data)

    def send_signed_transaction(self, signed_transaction: str) -> Transaction:
        """Send a signed transaction."""
        data = self._make_request("POST", "/transactions/send", json_data={
            "signed_transaction": signed_transaction
        })
        return Transaction(**data)

    def estimate_fee(self, request: CreateTransactionRequest) -> FeeEstimate:
        """Estimate transaction fee."""
        data = self._make_request("POST", "/transactions/estimate-fee", json_data=request.__dict__)
        return FeeEstimate(**data)

    def get_transactions(
        self, 
        page: int = 1, 
        page_size: int = 20,
        address: Optional[str] = None
    ) -> PaginatedResponse[Transaction]:
        """Get paginated list of transactions."""
        params = {"page": page, "page_size": page_size}
        
        endpoint = f"/addresses/{address}/transactions" if address else "/transactions"
        data = self._make_request("GET", endpoint, params=params)
        
        return PaginatedResponse(
            items=[Transaction(**item) for item in data["items"]],
            total_count=data["total_count"],
            page=data["page"],
            page_size=data["page_size"],
            total_pages=data["total_pages"],
            has_next=data["has_next"],
            has_prev=data["has_prev"]
        )

    # Block Methods
    
    def get_latest_block(self) -> Block:
        """Get the latest block."""
        data = self._make_request("GET", "/blockchain/latest-block")
        return Block(**data)

    def get_block(self, height: int) -> Block:
        """Get block by height."""
        data = self._make_request("GET", f"/blockchain/blocks/{height}")
        return Block(**data)

    # ML Task Methods
    
    def create_ml_task(self, request: MLTaskRequest) -> MLTask:
        """Create a new ML task."""
        data = self._make_request("POST", "/ml-tasks", json_data=request.__dict__)
        return MLTask(**data)

    def get_ml_task(self, task_id: str) -> MLTask:
        """Get ML task by ID."""
        data = self._make_request("GET", f"/ml-tasks/{task_id}")
        return MLTask(**data)

    def get_ml_tasks(
        self, 
        page: int = 1, 
        page_size: int = 20,
        status: Optional[str] = None
    ) -> PaginatedResponse[MLTask]:
        """Get paginated list of ML tasks."""
        params = {"page": page, "page_size": page_size}
        if status:
            params["status"] = status
            
        data = self._make_request("GET", "/ml-tasks", params=params)
        
        return PaginatedResponse(
            items=[MLTask(**item) for item in data["items"]],
            total_count=data["total_count"],
            page=data["page"],
            page_size=data["page_size"], 
            total_pages=data["total_pages"],
            has_next=data["has_next"],
            has_prev=data["has_prev"]
        )

    # Governance Methods
    
    def create_proposal(self, request: CreateProposalRequest) -> Proposal:
        """Create a new governance proposal."""
        data = self._make_request("POST", "/governance/proposals", json_data=request.__dict__)
        return Proposal(**data)

    def get_proposal(self, proposal_id: str) -> Proposal:
        """Get proposal by ID."""
        data = self._make_request("GET", f"/governance/proposals/{proposal_id}")
        return Proposal(**data)

    def vote(self, proposal_id: str, option: str) -> None:
        """Vote on a proposal."""
        self._make_request("POST", f"/governance/proposals/{proposal_id}/vote", json_data={
            "option": option
        })

    # Utility Methods
    
    def set_api_key(self, api_key: str) -> None:
        """Update API key."""
        self.config.api_key = api_key
        self._client.headers["X-API-Key"] = api_key
        self._async_client.headers["X-API-Key"] = api_key

    def close(self) -> None:
        """Close HTTP clients."""
        self._client.close()

    async def aclose(self) -> None:
        """Close async HTTP client."""
        await self._async_client.aclose()

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.aclose()


class RateLimiter:
    """Simple rate limiter implementation."""
    
    def __init__(self, requests: int, window: int):
        self.requests = requests
        self.window = window
        self.calls = []
        
    def wait_if_needed(self) -> None:
        """Wait if rate limit would be exceeded."""
        now = time.time()
        self.calls = [call_time for call_time in self.calls if now - call_time < self.window]
        
        if len(self.calls) >= self.requests:
            sleep_time = self.window - (now - self.calls[0])
            if sleep_time > 0:
                time.sleep(sleep_time)
                
        self.calls.append(now)
        
    async def async_wait_if_needed(self) -> None:
        """Wait if rate limit would be exceeded (async)."""
        now = time.time()
        self.calls = [call_time for call_time in self.calls if now - call_time < self.window]
        
        if len(self.calls) >= self.requests:
            sleep_time = self.window - (now - self.calls[0])
            if sleep_time > 0:
                await asyncio.sleep(sleep_time)
                
        self.calls.append(now)