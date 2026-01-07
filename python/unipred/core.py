try:
    from .unipred_py import UnipredCore as _UnipredCore
except ImportError:
    _UnipredCore = None

from .market_quote_pb2 import MarketQuote, FetchedMarketList

class UnipredCore:
    def __init__(self, config: str) -> None:
        if _UnipredCore is None:
            raise ImportError("Failed to load native extension unipred_py")
        self._inner = _UnipredCore(config)

    def login(self, email: str, password: str) -> None:
        """
        Authenticates using email and password.
        """
        self._inner.login(email, password)

    def login_apikey(self, key_id: str, private_key_path: str) -> None:
        """
        Authenticates using an API key and private key file.
        """
        self._inner.login_apikey(key_id, private_key_path)

    def fetch_markets(
        self,
        exchange: str | None = None,
        limit: int = 100,
        cursor: str | None = None,
        status: str | None = None,
    ) -> FetchedMarketList:
        """
        Fetches markets with filtering options.
        """
        bytes_data = self._inner._fetch_markets_bytes(
            exchange, limit, cursor, status
        )
        market_list = FetchedMarketList()
        market_list.ParseFromString(bytes_data)
        return market_list

    def get_quote(self, ticker: str, exchange: str | None = None) -> MarketQuote:
        """
        Fetches a quote for the given ticker.
        
        Returns:
            MarketQuote: A Protobuf object containing the quote details.
        """
        bytes_data = self._inner._get_quote_bytes(ticker, exchange)
        quote = MarketQuote()
        quote.ParseFromString(bytes_data)
        return quote

    def ingest_all(
        self,
        db_path: str,
        lancedb_path: str,
        exchanges: list[str] | None = None,
        statuses: list[str] | None = None,
    ) -> None:
        """
        Ingests markets from configured exchanges into DuckDB and LanceDB.

        Args:
            db_path: Path to the DuckDB database file.
            lancedb_path: Path to the LanceDB directory.
            exchanges: List of exchanges to scrape (e.g. ["kalshi", "polymarket"]).
            statuses: List of statuses to filter (e.g. ["active", "closed"]).
        """
        self._inner.ingest_all(db_path, lancedb_path, exchanges, statuses)