import pytest
from unipred.market_quote_pb2 import FetchedMarketList

class TestMarkets:
    """Integration tests for fetching markets from exchanges."""

    def test_fetch_kalshi_markets(self, core):
        """Test fetching markets from Kalshi explicitly."""
        limit = 10
        market_list = core.fetch_markets(
            exchange="kalshi",
            limit=limit,
            status="open"
        )
        
        assert isinstance(market_list, FetchedMarketList)
        assert len(market_list.markets) > 0
        assert len(market_list.markets) <= limit
        
        # Validate structure
        market = market_list.markets[0]
        assert market.ticker
        assert market.title
        assert market.source == "Kalshi"
        assert market.status == "active"

    def test_fetch_polymarket_markets(self, core):
        """Test fetching markets from Polymarket explicitly."""
        # Polymarket pagination is different, but our wrapper standardizes the return format
        market_list = core.fetch_markets(
            exchange="polymarket",
            limit=10, # Polymarket limit logic is complex, checking basic fetch
        )
        
        assert isinstance(market_list, FetchedMarketList)
        assert len(market_list.markets) > 0
        
        # Validate structure
        market = market_list.markets[0]
        assert market.ticker
        assert market.title
        assert market.source == "Polymarket"
        # Polymarket active status is boolean true/false mapped to string
        assert market.status in ["active", "closed"]

    def test_fetch_markets_pagination(self, core):
        """Test pagination for Kalshi (since it's robustly supported)."""
        limit = 5
        
        # First page
        list1 = core.fetch_markets(exchange="kalshi", limit=limit, status="open")
        assert len(list1.markets) == limit
        assert list1.cursor
        
        # Second page
        list2 = core.fetch_markets(exchange="kalshi", limit=limit, cursor=list1.cursor, status="open")
        assert len(list2.markets) > 0
        
        tickers1 = {m.ticker for m in list1.markets}
        tickers2 = {m.ticker for m in list2.markets}
        assert tickers1.isdisjoint(tickers2)