import os
import sys
import argparse
import duckdb
import pandas as pd
import datetime
from typing import List, Optional
from sentence_transformers import SentenceTransformer
from sklearn.metrics.pairwise import cosine_similarity
from unipred import UnipredCore

# Database file path
DB_PATH = "new_markets.db"

def init_db(con: duckdb.DuckDBPyConnection):
    """Initialize the markets table."""
    con.execute("""
        CREATE TABLE IF NOT EXISTS markets (
            ticker VARCHAR,
            title VARCHAR,
            description VARCHAR,
            outcomes VARCHAR,
            source VARCHAR,
            status VARCHAR,
            ingested_at TIMESTAMP,
            PRIMARY KEY (ticker, source)
        )
    """)

def ingest_markets(core: UnipredCore, con: duckdb.DuckDBPyConnection, exchange: str, status: Optional[str] = None):
    """Fetch markets from an exchange and store them in DuckDB."""
    print(f"Ingesting markets from {exchange} (status={status})...")
    
    cursor = None
    total_ingested = 0
    batch_size = 100
    
    while True:
        try:
            # Fetch markets using the python wrapper
            # Note: The underlying rust implementation maps None status differently for each exchange.
            response = core.fetch_markets(exchange=exchange, limit=batch_size, cursor=cursor, status=status)
            
            if not response.markets:
                break
                
            # Prepare data for insertion
            data = []
            now = datetime.datetime.now()
            for m in response.markets:
                # Clean title: sometimes it might be None or empty
                title = m.title if m.title else "Unknown"
                desc = m.description if m.description else ""
                outcomes = ", ".join(m.outcomes) if m.outcomes else ""
                data.append((m.ticker, title, desc, outcomes, m.source, m.status, now))
            
            # Upsert into DuckDB
            # We use INSERT OR REPLACE to update existing entries
            con.executemany("""
                INSERT OR REPLACE INTO markets (ticker, title, description, outcomes, source, status, ingested_at)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """, data)
            
            count = len(data)
            total_ingested += count
            print(f"  Fetched {count} markets. Total: {total_ingested}")
            
            # Update cursor for next batch
            if response.cursor:
                cursor = response.cursor
            else:
                break

            # Safety break for development to avoid infinite loops if API misbehaves
            # or massive data loads if not intended.
            if total_ingested > 5000:
                print("  Reached safety limit of 5000 markets per ingestion call.")
                break
                
        except Exception as e:
            print(f"Error fetching from {exchange}: {e}")
            break
            
    print(f"Finished ingesting {exchange}.")

def correlate_markets(con: duckdb.DuckDBPyConnection, threshold: float = 0.75):
    """Load markets, generate embeddings, and find correlations."""
    print("Loading markets from database...")
    df = con.execute("SELECT ticker, title, description, outcomes, source FROM markets").fetchdf()
    
    if df.empty:
        print("No markets found in database.")
        return

    print(f"Loaded {len(df)} markets.")
    
    # Separate by source to cross-correlate
    # We want to find matches between Kalshi and Polymarket
    kalshi_df = df[df['source'] == 'Kalshi'].reset_index(drop=True)
    poly_df = df[df['source'] == 'Polymarket'].reset_index(drop=True)
    
    if kalshi_df.empty or poly_df.empty:
        print("Need markets from both Kalshi and Polymarket to correlate.")
        return

    print("Generating embeddings (this may take a moment)...")
    # Use a lightweight but effective model
    model = SentenceTransformer('all-MiniLM-L6-v2')
    
    def make_rich_text(row):
        parts = [f"Title: {row['title']}"]
        if row['description']:
            parts.append(f"Description: {row['description']}")
        if row['outcomes']:
            parts.append(f"Outcomes: {row['outcomes']}")
        return "\n".join(parts)

    kalshi_texts = kalshi_df.apply(make_rich_text, axis=1).tolist()
    poly_texts = poly_df.apply(make_rich_text, axis=1).tolist()
    
    kalshi_embeddings = model.encode(kalshi_texts)
    poly_embeddings = model.encode(poly_texts)
    
    print("Calculating similarity matrix...")
    similarity_matrix = cosine_similarity(kalshi_embeddings, poly_embeddings)
    
    print(f"Finding matches with similarity > {threshold}...")
    matches = []
    
    for i in range(len(kalshi_df)):
        for j in range(len(poly_df)):
            score = similarity_matrix[i][j]
            if score > threshold:
                matches.append({
                    'score': score,
                    'kalshi_ticker': kalshi_df.iloc[i]['ticker'],
                    'kalshi_title': kalshi_df.iloc[i]['title'],
                    'poly_ticker': poly_df.iloc[j]['ticker'],
                    'poly_title': poly_df.iloc[j]['title']
                })
    
    # Sort by score descending
    matches.sort(key=lambda x: x['score'], reverse=True)
    
    # Output results
    print(f"\nFound {len(matches)} correlations:")
    print("-" * 80)
    print(f"{'Score':<6} | {'Kalshi Title':<35} | {'Polymarket Title':<35}")
    print("-" * 80)
    
    for m in matches[:50]:  # Show top 50
        k_title = (m['kalshi_title'][:32] + '..') if len(m['kalshi_title']) > 32 else m['kalshi_title']
        p_title = (m['poly_title'][:32] + '..') if len(m['poly_title']) > 32 else m['poly_title']
        print(f"{m['score']:.4f} | {k_title:<35} | {p_title:<35}")

def export_to_s3(con: duckdb.DuckDBPyConnection, bucket: str, path: str):
    """Export the markets table to S3 (Parquet format)."""
    print(f"Exporting markets to s3://{bucket}/{path}...")
    
    # Configure S3 credentials (assumes AWS env vars or ~/.aws/credentials are present)
    try:
        con.execute("INSTALL httpfs;")
        con.execute("LOAD httpfs;")
        # Note: If credentials aren't picked up automatically, you can set them:
        # con.execute("SET s3_access_key_id='...';")
        # con.execute("SET s3_secret_access_key='...';")
        
        # Export to Parquet using the s3:// protocol
        query = f"COPY markets TO 's3://{bucket}/{path}' (FORMAT PARQUET);"
        con.execute(query)
        print("Export complete.")
    except Exception as e:
        print(f"Error exporting to S3: {e}")
        print("Ensure AWS credentials are set and httpfs extension is installable.")

def main():
    parser = argparse.ArgumentParser(description="Ingest and correlate prediction markets.")
    parser.add_argument('action', choices=['ingest', 'correlate', 'all'], help="Action to perform")
    parser.add_argument('--threshold', type=float, default=0.75, help="Correlation threshold (0.0-1.0)")
    parser.add_argument('--s3-bucket', type=str, help="S3 bucket for export (optional)")
    parser.add_argument('--s3-path', type=str, default="markets.parquet", help="S3 path/key for export")
    args = parser.parse_args()

    # Initialize Core
    try:
        core = UnipredCore("env")
    except ImportError:
        print("Could not import UnipredCore. Ensure the native extension is built.")
        sys.exit(1)

    # Initialize DuckDB
    con = duckdb.connect(DB_PATH)
    init_db(con)

    if args.action in ['ingest', 'all']:
        # Ingest Kalshi
        # Try fetching default (usually open) and explicitly closed if supported.
        ingest_markets(core, con, "kalshi", status="active")
        ingest_markets(core, con, "kalshi", status="closed")
        
        # Ingest Polymarket
        ingest_markets(core, con, "polymarket")

    if args.action in ['correlate', 'all']:
        correlate_markets(con, threshold=args.threshold)

    if args.s3_bucket:
        export_to_s3(con, args.s3_bucket, args.s3_path)

    con.close()

if __name__ == "__main__":
    main()