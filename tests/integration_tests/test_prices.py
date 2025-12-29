import pytest
# All these should be valid tickers that should return something.
# If they don't due to market being deleted etc, remove them.
GOOD_TICKERS = [
    "KXLINKMINY-25-10", "KXELONMARS-99", "KXCOLONIZEMARS-50", "KXNEWPOPE-70-PPIZ",
    #"0xd903891c2b9046cae14615afc4c5245370143503f7b2dfc13919acee07a1696d"
]
BAD_TICKERS = ["KX_ASDASASXZX!@3",
    #"0xd903891c2b9046cae14615afc4c5245370143503f7b2dfc13919acee07a1696x"
]

class TestPrices:

    @pytest.mark.parametrize(
        ("ticker"),
        GOOD_TICKERS
    )
    def test_good_tickers(self, core, ticker):
        assert core.get_quote(ticker)

    @pytest.mark.parametrize(
        ("ticker"),
        BAD_TICKERS
    )
    def test_bad_tickers(self, core, ticker):
        with pytest.raises(Exception): # todo scope to specific exception
            core.get_quote(ticker)
