from .market_quote_pb2 import FetchedMarketList

class UnipredCore:
    def __init__(self, config: str) -> None: ...
    def login(self, email: str, password: str) -> None: ...
    def login_apikey(self, key_id: str, private_key_path: str) -> None: ...
    def _fetch_markets_bytes(
        self,
        exchange: str | None = None,
        limit: int = 100,
        cursor: str | None = None,
        status: str | None = None,
    ) -> bytes: ...
    def fetch_markets(
        self,
        exchange: str | None = None,
        limit: int = 100,
        cursor: str | None = None,
        status: str | None = None,
    ) -> FetchedMarketList: ...
    def _get_quote_bytes(self, ticker: str, exchange: str | None = None) -> bytes: ...