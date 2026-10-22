# Performance budget

[Docs](../README.md) › [Architecture](README.md)

| Measure | Target |
| --- | --- |
| Agent CPU when idle | Under 1% |
| Agent RAM | Under 60 MB |
| Desktop app RAM (open) | Under 200 MB |
| First useful results | Under 10 seconds |
| Full first scan (without deep disk scan) | Under 60 seconds |
| Database size | Under 200 MB |

The project should add an automated check that fails the build if these limits are broken.
