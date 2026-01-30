## Performance Considerations

### File Operations

- Use memory-mapped files for large archives
- Batch symlink operations
- Async file I/O with tokio

### Database

- Use SQLite WAL mode for concurrent reads
- Prepare statements for repeated queries
- Batch inserts with transactions

### UI

- Virtual scrolling for large mod lists
- Lazy loading of mod metadata
- Background indexing
