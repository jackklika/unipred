import os
import json
import duckdb
import tempfile
from datetime import datetime, timezone
from dotenv import load_dotenv
from unipred.unipred_py import UnipredCore

def main():
    load_dotenv()

    # 1. Setup configuration
    key_id = os.getenv("KALSHI_API_KEY_ID")
    private_key = os.getenv("KALSHI_PRIVATE_KEY")

    if not key_id or not private_key:
        print("Please set KALSHI_API_KEY_ID and KALSHI_PRIVATE_KEY environment variables.")
        return

    # 2. Setup Unipred
    # The config string is currently unused by the core implementation
    core = UnipredCore("{}")

    # Create temporary file for private key since binding expects a path
    with tempfile.NamedTemporaryFile(mode='w', delete=False) as f:
        f.write(private_key)
        temp_key_path = f.name

    print(f"Logging in with API Key ID: {key_id}...")
    try:
        core.login_apikey(key_id, temp_key_path)
        print("Login successful.")
    except Exception as e:
        print(f"Login failed: {e}")
        if os.path.exists(temp_key_path):
            os.unlink(temp_key_path)
        return

    if os.path.exists(temp_key_path):
        os.unlink(temp_key_path)

    # 3. Define Time Range
    # "Close by Jan 2nd". Assuming Jan 2nd 2026 based on current late 2025 date.
    target_date = datetime(2026, 1, 2, 23, 59, 59, tzinfo=timezone.utc)
    max_ts = int(target_date.timestamp())

    print(f"Fetching open markets closing before {target_date} (ts: {max_ts})")

    # 4. Fetch Loop
    cursor = None
    all_markets = []
    page_count = 0

    while True:
        print(f"Fetching page {page_count + 1}...", end=" ", flush=True)
        try:
            response_json = core.fetch_kalshi_markets(
                limit=100,
                cursor=cursor,
                status="open",
                max_close_ts=max_ts
            )
        except Exception as e:
            print(f"\nError fetching markets: {e}")
            break

        data = json.loads(response_json)
        markets = data.get("markets", [])
        next_cursor = data.get("cursor")

        count = len(markets) if markets else 0
        print(f"Got {count} markets.")

        if markets:
            all_markets.extend(markets)

        cursor = next_cursor
        if not cursor:
            break

        page_count += 1

    print(f"Total markets fetched: {len(all_markets)}")

    if not all_markets:
        print("No markets found to save.")
        return

    # 5. Store in DuckDB
    db_file = "kalshi_markets.db"
    con = duckdb.connect(db_file)
    print(f"Connected to DuckDB: {db_file}")

    # Use a temporary JSON file to bulk load into DuckDB
    # This allows DuckDB to auto-infer the schema from the complex nested objects
    temp_file = "temp_markets_dump.json"
    try:
        with open(temp_file, "w") as f:
            json.dump(all_markets, f)

        # Drop table if exists to start fresh
        con.execute("DROP TABLE IF EXISTS raw_markets")

        # Load JSON
        print("Loading data into table 'raw_markets'...")
        con.execute(f"CREATE TABLE raw_markets AS SELECT * FROM read_json_auto('{temp_file}')")

        # Verify
        count = con.execute("SELECT count(*) FROM raw_markets").fetchone()[0]
        print(f"Successfully stored {count} rows in 'raw_markets'.")

        # 6. Basic Analysis
        print("\n--- Top 5 Markets by Volume ---")
        con.sql("""
            SELECT
                ticker,
                title,
                last_price,
                volume,
                yes_bid,
                yes_ask
            FROM raw_markets
            ORDER BY volume DESC
            LIMIT 5
        """).show()

    except Exception as e:
        print(f"Database error: {e}")
    finally:
        if os.path.exists(temp_file):
            os.remove(temp_file)
        con.close()

if __name__ == "__main__":
    main()

if __name__ == "__main__":
    main()