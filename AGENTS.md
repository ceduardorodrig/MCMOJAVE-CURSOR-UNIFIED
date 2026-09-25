---
tags: [meta, agents, governance, linux, wayland]
---

# AGENTS.md — MCMOJAVE-CURSOR-UNIFIED Governance Rules

Este repositório contém o projeto **MCMOJAVE-CURSOR-UNIFIED**, um tema de cursor dual-spec (Hyprcursor vetorial + XCursor multi-resolução calibrado 1:1) com ferramenta de compilação em Rust nativo.

Ao modificar qualquer arquivo deste repositório, siga estas regras obrigatórias de governança:

## 🦀 Padrões Rust & Integridade

1. **RUST NATIVO & SOBERANIA (RUST 2024)** — Todo código deve permanecer em Rust compilado nativo (`edition = "2024"`). Proibido scripts intermediários Python ou Bash legados (`ARCH-NO-PYTHON`).

2. **PROIBIÇÃO DE UNWRAP/EXPECT EM PRODUÇÃO (`RUST-NO-UNWRAP`)** — Tratamento de erros deve ser determinístico usando `?`, `match` ou fallback seguro (`unwrap_or`). O uso de `unwrap()` ou `expect()` causa pânico incondicional e é categorizado como tentativa de bypass.

3. **VERIFICAÇÃO OBRIGATÓRIA DO STÊNIOSENTINEL (REGRA 0)** — Antes de qualquer commit ou conclusão de turno, é obrigatório executar `stenio --path .`. O Quality Gate deve aprovar com zero erros bloqueantes.

4. **SEGURANÇA & ZERO SEGREDOS (`SEC-SECRETS`)** — Nenhuma credencial ou token deve ser adicionado ao código.

5. **DISCLAIMER PADRONIZADO NO README (`DOC-VIBE-DISCLAIMER`)** — O `README.md` raiz deve manter o disclaimer padronizado de governança humana-IA:
   ```markdown
   <div align="center">

   > **Yes... This is a Vibe Coded project**
   >
   > Governed by 🤖 **StenioSentinel** (our Rust-based AI Governance Sentinel) with **Carlos Eduardo Rodrigues** ([@ceduardorodrig](https://github.com/ceduardorodrig)).

   </div>
   ```
