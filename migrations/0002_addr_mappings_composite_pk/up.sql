-- Remove exact duplicates prior to adding a unique constraint
WITH ranked AS (
    SELECT ctid,
           ROW_NUMBER() OVER (PARTITION BY t_address, q_address ORDER BY t_address) AS rn
    FROM addr_mappings
)
DELETE FROM addr_mappings a
USING ranked r
WHERE a.ctid = r.ctid
  AND r.rn > 1;

-- Add unique index to support ON CONFLICT (t_address, q_address)
CREATE UNIQUE INDEX IF NOT EXISTS idx_addr_mappings_t_q_unique
ON addr_mappings (t_address, q_address);
