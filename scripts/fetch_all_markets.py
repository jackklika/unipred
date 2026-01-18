import os
import sys
import argparse
import tempfile
from dotenv import load_dotenv
from unipred import UnipredCore

def main():
    load_dotenv()

    parser = argparse.ArgumentParser(description="Ingest all markets into DuckDB and LanceDB.")
    parser.add_argument("--db-path", default="markets.db", help="Path to DuckDB file")
    parser.add_argument("--lancedb-path", default="lancedb_data", help="Path to LanceDB directory")
    parser.add_argument("--exchange", action="append", help="Exchange to scrape (can be specified multiple times). Default: all.")
    parser.add_argument("--test", action="store_true", help="Run in test mode (limit pages)")
    args = parser.parse_args()

    # 1. Initialize Core
    try:
        # Config string is currently unused by Rust core but required by constructor
        core = UnipredCore("env")
    except ImportError:
        print("Could not import UnipredCore. Ensure the native extension is built.")
        sys.exit(1)

    # 2. Authentication (Required for Kalshi)
    key_id = os.getenv("KALSHI_API_KEY_ID")
    private_key = os.getenv("KALSHI_PRIVATE_KEY")

    if key_id and private_key:
        print(f"Logging in to Kalshi with Key ID: {key_id}...")
        # Create temporary file for private key since binding expects a path
        with tempfile.NamedTemporaryFile(mode='w', delete=False) as f:
            f.write(private_key)
            temp_key_path = f.name

        try:
            core.login_apikey(key_id, temp_key_path)
            print("Login successful.")
        except Exception as e:
            print(f"Login failed: {e}")
            sys.exit(1)
        finally:
            if os.path.exists(temp_key_path):
                os.unlink(temp_key_path)
    else:
        print("Warning: KALSHI_API_KEY_ID or KALSHI_PRIVATE_KEY not set. Kalshi fetch may fail or be limited.")

    # 3. Ingest
    print(f"Starting ingestion engine...")
    print(f"DuckDB: {args.db_path}")
    print(f"LanceDB: {args.lancedb_path}")

    exchanges = args.exchange if args.exchange else ["kalshi", "polymarket"]

    try:
        # Fetch everything: Kalshi (active/closed) and Polymarket
        core.ingest_all(
            db_path=args.db_path,
            lancedb_path=args.lancedb_path,
            exchanges=exchanges,
            statuses=["active", "closed"],
            test_mode=args.test
        )
        print("Ingestion complete.")
    except Exception as e:
        print(f"Ingestion failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
