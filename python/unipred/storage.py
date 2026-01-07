from dataclasses import dataclass
from typing import List, Any
try:
    from .unipred_py import PyLanceDb as _PyLanceDb
except ImportError:
    _PyLanceDb = None

@dataclass
class MarketEmbedding:
    """
    Python representation of a market with its embedding vector.
    Matches the structure expected by the Rust extension.
    """
    id: str
    vector: List[float]
    ticker: str
    source: str
    title: str
    description: str
    outcomes: str

class LanceStore:
    def __init__(self, uri: str):
        """
        Initialize a connection to LanceDB.
        
        Args:
            uri: Path to local directory (e.g. "./data/lancedb") or S3 URI (e.g. "s3://bucket/path")
        """
        if _PyLanceDb is None:
            raise ImportError("Failed to load native extension unipred_py")
        self._inner = _PyLanceDb.connect(uri)

    def add_markets(self, markets: List[MarketEmbedding]) -> None:
        """
        Add markets with embeddings to the store.
        """
        self._inner.add_markets(markets)

    def create_index(self) -> None:
        """
        Create an IVF-PQ index for fast similarity search.
        """
        self._inner.create_index()

    def search(self, query_vector: List[float], limit: int = 10) -> List[dict]:
        """
        Search for similar markets.
        Returns a list of dictionaries with market metadata.
        """
        # The Rust extension returns a list of tuples:
        # (id, ticker, source, title, description, outcomes)
        results = self._inner.search(query_vector, limit)
        
        return [
            {
                "id": r[0],
                "ticker": r[1],
                "source": r[2],
                "title": r[3],
                "description": r[4],
                "outcomes": r[5]
            }
            for r in results
        ]