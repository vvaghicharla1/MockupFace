# Contributing to Mockupface

Thank you for your interest in contributing. This document covers how to get set up and what to keep in mind when submitting changes.

---

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/mockupface.git`
3. Follow the installation steps in [README.md](README.md)
4. Create a feature branch: `git checkout -b feature/your-feature-name`

---

## Development Setup

### Backend (Rust)

```bash
cd backend
cp .env.example .env   # fill in your keys
cargo run              # starts on :8080
cargo test             # run tests
cargo clippy           # linter
cargo fmt              # formatter
```

### Frontend (React)

```bash
cd frontend
npm install
npm run dev            # starts on :5173 with /api proxy
npm run build          # production build
```

---

## Code Style

**Rust**
- Run `cargo fmt` before committing
- No clippy warnings (`cargo clippy -- -D warnings`)
- All public functions should have doc comments

**JavaScript / React**
- Functional components only
- Props typed via JSDoc where helpful
- Keep components focused — one responsibility per file

---

## Submitting a Pull Request

1. Make sure the backend compiles: `cargo build`
2. Make sure the frontend builds: `npm run build`
3. Write a clear PR description:
   - What problem does this solve?
   - How did you test it?
   - Any breaking changes?
4. Reference any related issues

---

## Areas Where Help Is Welcome

- Additional platform support (Shopify, Redbubble, eBay)
- User feedback loop for RAG quality improvement
- Batch processing multiple products
- Test coverage for Rust routes
- Frontend accessibility improvements
- Docker / docker-compose setup

---

## Reporting Issues

Open a GitHub issue with:
- Your OS and Rust/Node versions
- Steps to reproduce
- Expected vs actual behaviour
- Relevant logs or error messages

---

## Questions

Open a GitHub Discussion or reach out via the links in README.md.
