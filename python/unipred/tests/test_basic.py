import unipred

def test_hello():
    result = unipred.hello()
    assert "Hello from unipred!" in result
    assert "Executing with config: default_config" in result

def test_core_direct():
    from unipred.unipred_py import UnipredCore
    core = UnipredCore("custom_config")
    assert core.execute() == "Executing with config: custom_config"
