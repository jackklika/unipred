from google.protobuf.message import Message
from typing import ClassVar, Optional, Iterable

class MarketQuote(Message):
    ticker: str
    source: str
    price: str
    bid: str
    ask: str
    volume: str
    timestamp: str
    def __init__(
        self,
        ticker: Optional[str] = ...,
        source: Optional[str] = ...,
        price: Optional[str] = ...,
        bid: Optional[str] = ...,
        ask: Optional[str] = ...,
        volume: Optional[str] = ...,
        timestamp: Optional[str] = ...,
    ) -> None: ...

class FetchedMarket(Message):
    ticker: str
    title: str
    source: str
    status: str
    def __init__(
        self,
        ticker: Optional[str] = ...,
        title: Optional[str] = ...,
        source: Optional[str] = ...,
        status: Optional[str] = ...,
    ) -> None: ...

class FetchedMarketList(Message):
    cursor: str
    markets: Iterable[FetchedMarket]
    def __init__(
        self,
        cursor: Optional[str] = ...,
        markets: Optional[Iterable[FetchedMarket]] = ...,
    ) -> None: ...