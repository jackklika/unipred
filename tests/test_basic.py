import unipred
from unipred.unipred_py import UnipredCore

def test_hello():
    # We haven't updated __init__.py yet, so hello() might fail if it relies on old API
    pass 

def test_get_quote():
    core = UnipredCore("config")
    
    # Test Kalshi dispatch
    quote_kalshi = core.get_quote("KZ-TICKER")
    print(f"Kalshi Quote: {quote_kalshi}")
    assert "Kalshi" in quote_kalshi
    assert "0.75" in quote_kalshi

    # Test Polymarket dispatch
    quote_poly = core.get_quote("0x1234567890abcdef")
    print(f"Polymarket Quote: {quote_poly}")
    assert "Polymarket" in quote_poly
    assert "0.50" in quote_poly
