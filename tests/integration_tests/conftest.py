import os
import pytest
import tempfile
from collections.abc import Generator
from dotenv import load_dotenv
from unipred.unipred_py import UnipredCore

# Load environment variables from .env file if present
load_dotenv()

@pytest.fixture(scope="session")
def kalshi_credentials():
    """
    Retrieves Kalshi API Key ID and Private Key from environment variables.
    Skips tests if credentials are not found.
    """
    key_id = os.getenv("KALSHI_API_KEY_ID")
    private_key = os.getenv("KALSHI_PRIVATE_KEY")
    
    if not key_id or not private_key:
        pytest.skip("Skipping integration tests: KALSHI_API_KEY_ID and KALSHI_PRIVATE_KEY environment variables must be set.")
        
    return key_id, private_key

@pytest.fixture(scope="function")
def core(kalshi_credentials: tuple[str, str]) -> Generator[UnipredCore, None, None]:
    """
    Provides an authenticated UnipredCore instance.
    Handles the creation and cleanup of the temporary private key file required by the Rust client.
    """
    key_id, private_key = kalshi_credentials
    
    # Create temporary file for private key since the Rust binding expects a file path
    with tempfile.NamedTemporaryFile(mode='w', delete=False) as f:
        f.write(private_key)
        temp_key_path = f.name
        
    try:
        client = UnipredCore("{}")
        client.login_apikey(key_id, temp_key_path)
        yield client
    finally:
        # Cleanup temporary file
        if os.path.exists(temp_key_path):
            os.unlink(temp_key_path)