# Health score

[Docs](../README.md) › [Architecture](README.md)

1. Each category starts with its share of the score (for example, Storage 15, Security 15, Hardware 15).
2. Each open finding deducts points: critical −15, warning −5, info 0. A category can never go below 0.
3. The total is out of 100, with a breakdown shown on tap: "Storage −10 (disk 94% full)".
4. The score only uses findings the system can actually check, so a missing sensor does not lower it.
