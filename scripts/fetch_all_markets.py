import os
import json
import time
import duckdb
import tempfile
import random
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
    
    # 3. Setup Persistence
    db_file = "kalshi_all_markets.db"
    con = duckdb.connect(db_file)
    print(f"Connected to DuckDB: {db_file}")
    
    existing_tickers = set()
    try:
        # Check if table exists
        con.execute("DESCRIBE raw_markets")
        print("Table 'raw_markets' exists. Checking for existing tickers...")
        rows = con.execute("SELECT ticker FROM raw_markets").fetchall()
        existing_tickers = {r[0] for r in rows}
        print(f"Found {len(existing_tickers)} existing markets in DB. These will be skipped (incremental load).")
    except Exception:
        print("Table 'raw_markets' does not exist. It will be created.")

    print("Fetching all open markets...")
    
    # 4. Fetch Loop
    cursor = None
    total_new_markets = 0
    page_count = 0
    
    while True:
        print(f"Fetching page {page_count + 1}...", end=" ", flush=True)
        
        response_json = None
        
        # Retry loop for single page
        retries = 0
        max_retries = 10
        success = False
        
        while retries < max_retries:
            try:
                response_json = core.fetch_kalshi_markets(
                    limit=100,
                    cursor=cursor,
                    status="open"
                )
                success = True
                break
            except Exception as e:
                err_str = str(e)
                # Check for rate limit or other transient errors
                if "429" in err_str or "Too Many Requests" in err_str or "500" in err_str or "502" in err_str:
                    wait_time = (2 ** retries) + random.uniform(0, 1)
                    print(f"\nAPI Error ({e}). Retrying in {wait_time:.2f}s...")
                    time.sleep(wait_time)
                    retries += 1
                else:
                    print(f"\nFatal error fetching markets: {e}")
                    # For unknown errors, we stop to avoid infinite loops on logic bugs
                    break
        
        if not success:
            print("\nMax retries reached or fatal error. Stopping fetch loop.")
            break
        
        # Process Data
        try:
            data = json.loads(response_json)
            markets = data.get("markets", [])
            next_cursor = data.get("cursor")
            
            count = len(markets) if markets else 0
            print(f"Got {count} markets.", end=" ")
            
            if markets:
                # Filter out markets we already have
                new_markets = [m for m in markets if m['ticker'] not in existing_tickers]
                
                if new_markets:
                    print(f"({len(new_markets)} new)")
                    
                    # To support incremental persistence, we write this batch to DB immediately.
                    temp_batch_file = f"temp_batch_{page_count}.json"
                    with open(temp_batch_file, "w") as f:
                        json.dump(new_markets, f)
                    
                    try:
                        # Check existence again (in case it was created in previous loop iteration)
                        table_exists = False
                        try:
                            con.execute("DESCRIBE raw_markets")
                            table_exists = True
                        except:
                            pass
                        
                        if not table_exists:
                            # Create table from this batch
                            con.execute(f"CREATE TABLE raw_markets AS SELECT * FROM read_json_auto('{temp_batch_file}')")
                        else:
                            # Insert into existing table
                            # We use read_json_auto to match schema as best as possible
                            con.execute(f"INSERT INTO raw_markets SELECT * FROM read_json_auto('{temp_batch_file}')")
                            
                        total_new_markets += len(new_markets)
                        
                        # Update our local set of known tickers
                        for m in new_markets:
                            existing_tickers.add(m['ticker'])
                        
                    except Exception as db_err:
                        print(f"\nDatabase error inserting batch: {db_err}")
                    finally:
                        if os.path.exists(temp_batch_file):
                            os.remove(temp_batch_file)
                else:
                    print("(All skipped)")

            cursor = next_cursor
            if not cursor:
                print("\nNo cursor returned. End of list.")
                break
                
            page_count += 1
            # Respectful pause
            time.sleep(0.1) 
            
        except Exception as parse_err:
            print(f"\nError processing response: {parse_err}")
            break

    print(f"\nTotal new markets added: {total_new_markets}")
    print(f"Total markets in DB: {len(existing_tickers)}")
    
    # 5. Basic Analysis
    try:
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
        print(f"Error querying DB: {e}")
        
    con.close()

if __name__ == "__main__":
    main()