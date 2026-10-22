# AI assistant

[Docs](../README.md) › [Architecture](README.md)

1. **Local first:** it uses Ollama with a small model and is off until the user turns it on.
2. **Read-only tools:** the AI can call agent functions like `get_findings`, `get_events(since)`, `get_boot_list`, and `get_logs(filter)`. It cannot run commands.
3. **Grounded answers:** each answer shows the data or log lines it used.
4. **Fixes go through the fix engine:** the AI can only suggest a known fix action ID, which then goes through preview and approval.
5. **Cloud option:** when enabled, DeeCure removes usernames, hostnames, IPs, and serial numbers before sending.
