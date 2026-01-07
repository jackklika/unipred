from .core import UnipredCore
from .market_quote_pb2 import MarketQuote, FetchedMarketList
from .storage import LanceStore, MarketEmbedding

__all__ = ["UnipredCore", "MarketQuote", "FetchedMarketList", "LanceStore", "MarketEmbedding"]

def hello() -> str:
    # Deprecated helper function kept for backwards compatibility
    try:
        from .unipred_py import UnipredCore as _UnipredCore
        if _UnipredCore:
            return "Hello from unipred!"
    except ImportError:
        pass
    return "Hello from unipred! Core not available."