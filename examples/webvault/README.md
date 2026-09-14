# Webvault

A source-only Ject CLI that saves HTTP snapshots. It exercises application
arguments, HTTP, binary files, JSON metadata, and directory operations without
a native extension.

From this directory:

```sh
ject run -- fetch https://example.com snapshots/example
ject run -- show snapshots/example
ject test
```

`fetch` creates a new destination directory containing `body.bin` (the exact
response bytes) and `meta.json` (final URL, status, content type, and byte count).
It refuses to overwrite an existing destination. `show` checks that the body
length still matches the metadata.

HTTP responses are limited to 16 MiB and read fully into memory. Webvault is a
small end-to-end example, not a streaming crawler or an archival integrity tool.
