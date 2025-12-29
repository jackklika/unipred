.maxrows 0
.mode column

SELECT
ticker,
  title,
  open_time,
  close_time,
  yes_ask,
  no_ask,
  LEAST(yes_ask, no_ask) AS best_ask,
  GREATEST(yes_ask, no_ask) AS worst_ask,
  volume_24h
FROM raw_markets
WHERE open_time < DATE '2025-10-01'
  AND close_time < TIMESTAMP '2026-01-03'
  AND LEAST(yes_ask, no_ask) <= 97
  AND GREATEST(yes_ask, no_ask) <= 97
ORDER BY best_ask ASC, volume_24h DESC;
