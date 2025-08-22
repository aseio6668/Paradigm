"""
Paradigm Python SDK

Official Python SDK for interacting with the Paradigm blockchain network.
Provides simple and intuitive APIs for blockchain operations, ML tasks,
governance, and cross-chain functionality.
"""

__version__ = "1.0.0"
__author__ = "Paradigm Network"
__email__ = "support@paradigm.network"

from .client import ParadigmClient
from .websocket import ParadigmWebSocket
from .wallet import ParadigmWallet
from .exceptions import (
    ParadigmError,
    NetworkError,
    ValidationError,
    AuthenticationError,
    RateLimitError,
)
from .types import (
    Transaction,
    Block,
    Account,
    MLTask,
    Proposal,
    NetworkStats,
    TransactionReceipt,
    FeeEstimate,
)
from .constants import NETWORKS, API_VERSION
from .utils import (
    validate_address,
    validate_amount,
    format_address,
    format_amount,
    parse_address,
    parse_amount,
)

__all__ = [
    # Main classes
    "ParadigmClient",
    "ParadigmWebSocket", 
    "ParadigmWallet",
    
    # Exceptions
    "ParadigmError",
    "NetworkError",
    "ValidationError",
    "AuthenticationError",
    "RateLimitError",
    
    # Types
    "Transaction",
    "Block",
    "Account",
    "MLTask", 
    "Proposal",
    "NetworkStats",
    "TransactionReceipt",
    "FeeEstimate",
    
    # Constants
    "NETWORKS",
    "API_VERSION",
    
    # Utilities
    "validate_address",
    "validate_amount",
    "format_address",
    "format_amount",
    "parse_address",
    "parse_amount",
]

# Package metadata
__title__ = "paradigm-sdk"
__description__ = "Official Python SDK for Paradigm blockchain network"
__url__ = "https://github.com/paradigm-network/paradigm-sdk-python"
__license__ = "MIT"