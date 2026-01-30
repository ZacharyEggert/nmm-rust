## Security Considerations

### Script Sandboxing

```rust
// WASM scripts run in a sandboxed environment
pub struct ScriptSandbox {
    engine: wasmtime::Engine,
    linker: wasmtime::Linker<ScriptState>,
}

impl ScriptSandbox {
    pub fn new() -> Self {
        let mut config = wasmtime::Config::new();
        // Limit memory and CPU
        config.max_wasm_stack(1024 * 1024);  // 1MB stack
        config.memory_guaranteed_dense_image_size(64 * 1024 * 1024);  // 64MB memory

        let engine = wasmtime::Engine::new(&config).unwrap();
        let mut linker = wasmtime::Linker::new(&engine);

        // Only expose safe APIs
        linker.func_wrap("env", "read_file", |path: &str| -> Vec<u8> {
            // Validate path is within mod archive
            // ...
        });

        Self { engine, linker }
    }
}
```

### Path Validation

```rust
pub fn validate_path(base: &Path, relative: &Path) -> Result<PathBuf, SecurityError> {
    let resolved = base.join(relative).canonicalize()?;

    // Ensure path doesn't escape base directory
    if !resolved.starts_with(base) {
        return Err(SecurityError::PathTraversal);
    }

    // Check for dangerous file names
    let file_name = resolved.file_name().unwrap_or_default();
    if DANGEROUS_FILES.contains(&file_name.to_string_lossy().as_ref()) {
        return Err(SecurityError::DangerousFile);
    }

    Ok(resolved)
}
```
