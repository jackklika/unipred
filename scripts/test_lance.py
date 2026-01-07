import shutil
import os
from sentence_transformers import SentenceTransformer
from unipred import LanceStore, MarketEmbedding

DB_PATH = "test_lance_db"

def main():
    # Clean up previous test run
    if os.path.exists(DB_PATH):
        shutil.rmtree(DB_PATH)

    print(f"Initializing LanceStore at {DB_PATH}...")
    store = LanceStore(DB_PATH)

    # 1. Create Mock Data
    markets = [
        {
            "ticker": "K-TRUMP-WIN",
            "source": "Kalshi",
            "title": "Donald Trump wins 2024 Election",
            "description": "Will Donald Trump be the winner...",
            "outcomes": "Yes, No"
        },
        {
            "ticker": "P-TRUMP-24",
            "source": "Polymarket",
            "title": "Presidential Election Winner 2024",
            "description": "Winner of the US Presidential election...",
            "outcomes": "Donald Trump, Kamala Harris"
        },
        {
            "ticker": "K-BTC-100K",
            "source": "Kalshi",
            "title": "Bitcoin hits $100k",
            "description": "Will Bitcoin trade above $100,000...",
            "outcomes": "Yes, No"
        }
    ]

    print("Generating embeddings...")
    model = SentenceTransformer('all-MiniLM-L6-v2')
    
    embeddings = []
    for m in markets:
        # Create rich text representation
        text = f"Title: {m['title']}\nDescription: {m['description']}\nOutcomes: {m['outcomes']}"
        vector = model.encode(text).tolist()
        
        embeddings.append(MarketEmbedding(
            id=f"{m['source']}:{m['ticker']}",
            vector=vector,
            ticker=m['ticker'],
            source=m['source'],
            title=m['title'],
            description=m['description'],
            outcomes=m['outcomes']
        ))

    print(f"Adding {len(embeddings)} markets to LanceDB...")
    store.add_markets(embeddings)

    # Skip index creation for small dataset
    # print("Creating index...")
    # store.create_index()

    # 2. Search
    search_text = "Who will be president?"
    print(f"\nSearching for: '{search_text}'")
    query_vector = model.encode(search_text).tolist()
    
    results = store.search(query_vector, limit=2)

    print("\nResults:")
    for r in results:
        print("-" * 40)
        print(f"ID:    {r['id']}")
        print(f"Title: {r['title']}")
        print(f"Desc:  {r['description']}")

    print("\nSuccess!")

if __name__ == "__main__":
    main()