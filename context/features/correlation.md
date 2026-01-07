What I want to do is load every market into a database, open and closed, polymarket and kalshi, and then correlate common markets. For example "Democratic nominee for President in 2028?" is represented by kalshi in https://kalshi.com/markets/kxpresnomd/democratic-primary-winner/kxpresnomd-28 and in polymarket at https://polymarket.com/event/democratic-presidential-nominee-2028. I would like to be able to assign a 'correlation score' to these.

Correlation could be based on their titles, their common strikes, common prices to strikes, etc.

How do you propose doing this? Think about how this could be done locally but putting data in a cloud datastore on AWS, while allowing for some local development. Maybe something with embeddings. Locally I'm using duckdb right now for some things.
