import pytest
from unipred.market_quote_pb2 import MarketQuote

# All these should be valid tickers that should return something.
# If they don't due to market being deleted etc, remove them.
GOOD_TICKERS = [
    ("KXLINKMINY-25-10", None),
    ("KXELONMARS-99", None),
    ("KXCOLONIZEMARS-50", None),
    ("KXNEWPOPE-70-PPIZ", None),
    ("36725157385158152303355940271421346899386884953712631735038848833359115722560", "polymarket")
]
BAD_TICKERS = ["KX_ASDASASXZX!@3",
    "0xd903891c2b9046cae14615afc4c5245370143503f7b2dfc13919acee07a1696x"
]

class TestPrices:

    @pytest.mark.parametrize(
        ("ticker", "exchange"),
        GOOD_TICKERS
    )
    def test_good_tickers(self, core, ticker, exchange):
        quote = core.get_quote(ticker, exchange=exchange)
        assert quote
        assert isinstance(quote, MarketQuote)

        assert quote.ticker == ticker
        assert quote.price
        assert quote.timestamp

    @pytest.mark.parametrize(
        ("ticker"),
        BAD_TICKERS
    )
    def test_bad_tickers(self, core, ticker):
        with pytest.raises(Exception): # todo scope to specific exception
            core.get_quote(ticker)
