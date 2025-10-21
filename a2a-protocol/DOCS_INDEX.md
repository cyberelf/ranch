# A2A Protocol Documentation Index

**Current Version:** v0.4.0  
**Last Updated:** October 20, 2025

Welcome to the `a2a-protocol` crate documentation! This index helps you find the right document for your needs.

---

## Quick Start

**New to A2A Protocol?**
1. Start with [README.md](README.md) - Overview and quick start guide
2. Try running `cargo run --example server` - See it in action
3. Read the [A2A v0.3.0 Specification](https://github.com/a2aproject/A2A)

**Migrating from v0.3.x?**
- See [MIGRATION_v0.4.md](MIGRATION_v0.4.md) - Complete migration guide

**Want to contribute?**
- Check [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md) - See what needs to be built
- Review [UNIMPLEMENTED_FEATURES.md](UNIMPLEMENTED_FEATURES.md) - Current limitations

---

## Core Documentation

### 📖 [README.md](README.md)
**Purpose:** Overview, installation, and quick start  
**Audience:** Everyone  
**Contents:**
- What is A2A Protocol
- Installation instructions
- Basic client/server examples
- Supported RPC methods
- Architecture overview

**When to read:** First stop for all users

---

### 🗺️ [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)
**Purpose:** Complete implementation plan with timelines  
**Audience:** Contributors, project planners, stakeholders  
**Contents:**
- Current state (v0.4.0 baseline)
- What's implemented vs missing
- Release roadmap (v0.5.0 through v1.0.0)
- Detailed tasks for each release
- Timeline estimates (Q4 2025 - Q4 2026)
- Risk management
- Success metrics

**When to read:**
- Planning contributions
- Understanding project direction
- Tracking progress

---

### 🚀 [MIGRATION_v0.4.md](MIGRATION_v0.4.md)
**Purpose:** Guide for upgrading from v0.3.x to v0.4.0  
**Audience:** Existing users of v0.3.x  
**Contents:**
- Breaking changes explained
- Before/after code examples
- Endpoint migration (REST → JSON-RPC)
- Health check alternatives
- Streaming workarounds
- Complete server/client examples

**When to read:**
- Upgrading from v0.3.x
- Understanding v0.4.0 changes
- Finding replacement patterns

---

### ⚠️ [UNIMPLEMENTED_FEATURES.md](UNIMPLEMENTED_FEATURES.md)
**Purpose:** Lists features not yet implemented from A2A v0.3.0 spec  
**Audience:** Users evaluating the crate, contributors  
**Contents:**
- Current limitations
- Missing spec features by category
- Impact on interoperability
- Workaround patterns
- Timeline for implementation
- Contribution opportunities

**When to read:**
- Evaluating if crate meets your needs
- Finding workarounds for missing features
- Looking for contribution opportunities

---

## API Documentation

### 🔧 Generated API Docs
```bash
cargo doc --open
```
**Contents:**
- Full API reference
- Module documentation
- Type definitions
- Function signatures
- Code examples in rustdoc

**When to use:** When coding, looking up specific APIs

---

## Examples

### 📁 examples/ Directory

**Available Examples:**

#### `examples/server.rs`
Demonstrates creating an A2A-compliant JSON-RPC 2.0 server

```bash
cargo run --example server
```

**More examples coming in future releases!**

---

## External Resources

### Specification
- **A2A v0.3.0 Spec:** https://github.com/a2aproject/A2A
- **JSON-RPC 2.0 Spec:** https://www.jsonrpc.org/specification
- **W3C SSE Spec:** https://html.spec.whatwg.org/multipage/server-sent-events.html

### Community
- **GitHub Issues:** Report bugs, request features
- **GitHub Discussions:** Ask questions, share ideas
- **Spec Community:** Engage with A2A protocol community

---

## Document Status

| Document | Status | Last Updated | Version |
|----------|--------|--------------|---------|
| README.md | ✅ Current | Oct 20, 2025 | v0.4.0 |
| IMPLEMENTATION_ROADMAP.md | ✅ Current | Oct 20, 2025 | v0.4.0 |
| MIGRATION_v0.4.md | ✅ Current | Oct 20, 2025 | v0.4.0 |
| UNIMPLEMENTED_FEATURES.md | ✅ Current | Oct 20, 2025 | v0.4.0 |

---

## Quick Reference: What to Read When

### "I want to use this crate"
→ [README.md](README.md)

### "How do I upgrade from v0.3.x?"
→ [MIGRATION_v0.4.md](MIGRATION_v0.4.md)

### "Does this support feature X?"
→ [UNIMPLEMENTED_FEATURES.md](UNIMPLEMENTED_FEATURES.md)

### "I want to contribute"
→ [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)

### "How do I use this specific API?"
→ `cargo doc --open`

### "I need a working example"
→ `examples/` directory

### "What's the official spec?"
→ https://github.com/a2aproject/A2A

---

## Document Maintenance

### Updating Documentation

When making changes:
1. Update the relevant document
2. Update "Last Updated" date
3. Update this index if adding/removing documents
4. Ensure examples still work

### Versioning

Documentation follows the crate version. When releasing:
- Update all version references
- Update "Last Updated" dates
- Archive old migration guides if needed
- Create new migration guide for breaking changes

---

**Maintained By:** a2a-protocol team  
**License:** MIT OR Apache-2.0
