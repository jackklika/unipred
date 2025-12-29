import json
import pytest
from datetime import datetime, timezone

class TestKalshiMarkets:
    """Integration tests for fetching and filtering Kalshi markets."""

    def test_fetch_markets(self, core):
        """Test that we can fetch live markets from Kalshi."""
        limit = 10
        response_json = core.fetch_kalshi_markets(
            limit=limit,
            status="open"
        )

        data = json.loads(response_json)
        markets = data.get("markets", [])

        assert len(markets) > 0
        assert len(markets) <= limit

        # Validate basic market structure
        market = markets[0]
        assert "ticker" in market
        assert "title" in market
        assert "status" in market
        # Kalshi returns 'active' for open markets in v2 API
        assert market["status"] == "active"

    def test_fetch_markets_pagination(self, core):
        """Test that pagination cursor works."""
        limit = 5

        # First page
        resp1 = core.fetch_kalshi_markets(limit=limit, status="open")
        data1 = json.loads(resp1)
        markets1 = data1.get("markets", [])
        cursor1 = data1.get("cursor")

        assert len(markets1) == limit
        assert cursor1 is not None

        # Second page
        resp2 = core.fetch_kalshi_markets(limit=limit, cursor=cursor1, status="open")
        data2 = json.loads(resp2)
        markets2 = data2.get("markets", [])

        assert len(markets2) > 0

        # Ensure we got different markets by comparing tickers
        tickers1 = {m["ticker"] for m in markets1}
        tickers2 = {m["ticker"] for m in markets2}
        assert tickers1.isdisjoint(tickers2)

    def test_fetch_markets_time_filter(self, core):
        """Test filtering markets by close time."""
        # Use a far future date to ensure we get some results but verify the filter logic applies
        target_date = datetime(2026, 1, 1, tzinfo=timezone.utc)
        max_ts = int(target_date.timestamp())

        response_json = core.fetch_kalshi_markets(
            limit=50,
            status="open",
            max_close_ts=max_ts
        )

        data = json.loads(response_json)
        markets = data.get("markets", [])

        # If we got results, verify they respect the filter
        if len(markets) > 0:
            for market in markets:
                close_time_str = market.get("close_time")
                # Parse close_time (e.g. 2024-12-31T23:59:00Z)
                # We replace Z with +00:00 to ensure fromisoformat handles it correctly as UTC
                close_ts = datetime.fromisoformat(close_time_str.replace("Z", "+00:00")).timestamp()
                assert close_ts <= max_ts
