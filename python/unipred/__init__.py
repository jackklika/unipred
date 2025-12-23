try:
    from .unipred_py import UnipredCore
except ImportError:
    UnipredCore = None

def hello() -> str:
    if UnipredCore:
        core = UnipredCore("default_config")
        return f"Hello from unipred! Core says: {core.execute()}"
    return "Hello from unipred! Core not available."
