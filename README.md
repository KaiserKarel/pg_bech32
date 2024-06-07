# Pg_bech32

pg_bech32 adds encoding and decoding support for bech32 to your Postgres database.

```sql
SELECT bech32_encode('union'::text, decode('644a2606654a7c0e70bf343ae6b828d3fe448447','hex'), 'bech32'::text)
---
union1v39zvpn9ff7quu9lxsawdwpg60lyfpz8pmhfey
```

- [x] Simple.
- [x] Based on Rust.
- [x] Build it with pgrx or nix.
