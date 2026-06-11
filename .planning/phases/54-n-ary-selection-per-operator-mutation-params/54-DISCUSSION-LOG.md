# Phase 54: N-ary Selection + Per-Operator Mutation Params - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-28
**Phase:** 54-n-ary-selection-per-operator-mutation-params
**Areas discussed:** Scope y orden de ejecución, Mutation params struct, N-ary selection return type

---

## Scope y orden de ejecución

| Option | Description | Selected |
|--------|-------------|----------|
| Wave 1: N-ary selection → Wave 2: Mutation params | Cambios independientes en paths distintos; menor riesgo de conflicto | ✓ |
| Un solo PR con todo | Diff más grande pero evita estado intermedio roto | |
| Mutation params primero | Limpia ga.rs antes de tocar selection chain | |

**User's choice:** Wave 1: N-ary selection → Wave 2: Mutation params

---

| Option | Description | Selected |
|--------|-------------|----------|
| No tocar factory_multi_parent | Phase 51 está shipped, factory_multi_parent funciona, dejarlo separado | |
| Unificar: N-ary selection alimenta multi-parent crossover | Si select devuelve Vec<Vec<usize>>, grupos de 3+ van directamente a factory_multi_parent | ✓ |

**User's choice:** Unificar — el GA loop despacha a `factory_multi_parent` cuando `group.len() > 2`

---

| Option | Description | Selected |
|--------|-------------|----------|
| N viene de CrossoverConfiguration.num_parents | UNDX/SPX/PCX ya tienen num_parents; selection factory lo lee de ahí | ✓ |
| N en SelectionConfiguration.group_size | Más explícito pero puede desincronizarse con num_parents | |

**User's choice:** N viene de `CrossoverConfiguration.num_parents`

---

## Mutation params struct

| Option | Description | Selected |
|--------|-------------|----------|
| Parámetros en las variantes del enum Mutation | `Mutation::Gaussian { sigma: f64 }`; trait queda como `mutate(&mut U, &Mutation)` | ✓ |
| MutationParams enum separado | Trait recibe `&MutationParams`; segundo enum independiente | |
| Structs en MutationConfiguration | Extiende campos dedicados existentes; factory interno sigue igual | |

**User's choice:** Parámetros inline en el enum `Mutation`

---

| Option | Description | Selected |
|--------|-------------|----------|
| Aceptar que Mutation deja de ser Copy | v3.0.0 ya es breaking; documentar en MIGRATION.md; Clone en lugar de Copy | ✓ |
| Mantener Copy con params en HashMap/MutationConfiguration | Evita perder Copy pero API más indirecta | |

**User's choice:** Aceptar pérdida de Copy — v3.0.0 breaking change

---

| Option | Description | Selected |
|--------|-------------|----------|
| Sí: `fn mutate(&self, individual: &mut U, mutation: &Mutation)` | Breaking change en el trait público; documentado en MIGRATION.md | ✓ |
| No: el trait no cambia, solo impls internos | Evita romper custom operator impls | |

**User's choice:** El trait público cambia — custom `MutationOperator` impls deben actualizarse

---

## N-ary selection return type

| Option | Description | Selected |
|--------|-------------|----------|
| `Vec<Vec<usize>>` siempre | Un tipo unificado; grupos de 2 para estándar, N para multi-parent | ✓ |
| Mantener `Vec<(usize,usize)>` + nuevo select_nary | No rompe custom impls pero dos métodos paralelos | |

**User's choice:** `Vec<Vec<usize>>` — tipo único unificado

---

| Option | Description | Selected |
|--------|-------------|----------|
| factory recibe num_parents desde CrossoverConfiguration | Automático; sin configuración extra para el usuario | ✓ |
| SelectionConfiguration tiene group_size: usize | Más aislado pero requiere sincronización manual | |

**User's choice:** factory lee `CrossoverConfiguration.num_parents`

---

| Option | Description | Selected |
|--------|-------------|----------|
| Sí: `fn select(..., num_parents: usize) -> Vec<Vec<usize>>` | Breaking change en trait público; documentado en MIGRATION.md | ✓ |
| No: trait no recibe num_parents | Evita breaking change en trait pero limita utilidad del N-ary | |

**User's choice:** El trait público cambia — custom `SelectionOperator` impls deben actualizarse

---

## Claude's Discretion

- Parámetros en los enum variants usan `Option<f64>` (no `f64`) — `None` = usar default del operador. Preserva ergonomía zero-config.
- `MutationConfiguration` retiene: `probability`, `probability_max`, `dynamic_mutation`, `probability_step`. Todo lo demás se mueve a enum variants.
- GA loop dispatch: `if group.len() == 2 { factory() } else { factory_multi_parent() }` — sin nueva abstracción.

## Deferred Ideas

- GP-specific observer hooks (`on_bloat_detected`) — sigue fuera de scope
- Groups de tamaño no uniforme en una sola llamada a select — no necesario actualmente
- Inline params en `Crossover` variants — posible fase futura si surge el mismo problema
