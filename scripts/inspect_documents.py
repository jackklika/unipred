import argparse
import duckdb
import os
import sys

def main():
    parser = argparse.ArgumentParser(description="Inspect documents used for embeddings.")
    parser.add_argument("--db-path", default="new_markets.db", help="Path to DuckDB file")
    parser.add_argument("--limit", type=int, default=5, help="Number of documents to show")
    parser.add_argument("--source", type=str, choices=["Kalshi", "Polymarket"], help="Filter by source")
    parser.add_argument("--ticker", type=str, help="Filter by ticker")
    parser.add_argument("--search", type=str, help="Search term for title or description")
    
    args = parser.parse_args()

    if not os.path.exists(args.db_path):
        print(f"Error: Database file '{args.db_path}' not found.")
        print("Run 'uv run python scripts/fetch_all_markets.py --db-path new_markets.db' first.")
        sys.exit(1)

    try:
        con = duckdb.connect(args.db_path)
        
        # Build query
        query = "SELECT ticker, source, title, description, outcomes FROM markets WHERE 1=1"
        params = []

        if args.source:
            query += " AND source = ?"
            params.append(args.source)
        
        if args.ticker:
            query += " AND ticker = ?"
            params.append(args.ticker)
            
        if args.search:
            query += " AND (title ILIKE ? OR description ILIKE ?)"
            params.append(f"%{args.search}%")
            params.append(f"%{args.search}%")

        query += " LIMIT ?"
        params.append(args.limit)

        results = con.execute(query, params).fetchall()

        if not results:
            print("No markets found matching criteria.")
            return

        print(f"Showing {len(results)} markets from '{args.db_path}':")
        print("=" * 80)

        for row in results:
            ticker, source, title, description, outcomes = row
            
            # Reconstruct the document text exactly as it was embedded in Rust
            # Logic matches: unipred-core/src/ingestion/mod.rs
            # format!("Title: {}\nDescription: {}\nOutcomes: {}", title, description, outcomes)
            
            # Note: 'outcomes' in DB is stored as a comma-separated string, 
            # which matches the .join(", ") result used in ingestion.
            
            doc_text = f"Title: {title}\nDescription: {description}\nOutcomes: {outcomes}"
            
            print(f"MARKET: {ticker} ({source})")
            print("-" * 80)
            print(doc_text)
            print("=" * 80)
            print()

        con.close()

    except Exception as e:
        print(f"Error inspecting database: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()