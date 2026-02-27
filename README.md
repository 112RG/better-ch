![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/112RG/better-ch?utm_source=oss&utm_medium=github&utm_campaign=112RG%2Fbetter-ch&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews)

# better-ch (Anypoint Runtime Manager - scaffold)

This repository is a scaffold for a Linux-only Rust GUI application that will connect to the Anypoint Platform (MuleSoft) to manage runtimes.

Quick start

- Build normal (non-GUI):

```bash
cargo build
```

- Run the scaffolded app (placeholder):

```bash
cargo run
```

- Run the gpui GUI example (Linux only, feature gate):

```bash
cargo run --example demo --features gui
```

Next steps

- Implement `src/auth.rs` client-credentials flow using `oauth2` and secure token storage (keyring).
- Implement `src/api.rs` with typed endpoints for Anypoint runtime management (generate from OpenAPI if available).
- Implement `src/ws.rs` using `tokio-tungstenite` for logs streaming.
- Implement gpui UI in `src/ui.rs` using `gpui-component` widgets and background tasks.
