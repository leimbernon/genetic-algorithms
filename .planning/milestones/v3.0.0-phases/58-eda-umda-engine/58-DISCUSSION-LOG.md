# Phase 58: EDA / UMDA Engine - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-04
**Phase:** 58-eda-umda-engine
**Areas discussed:** Chromosome scope, Parent selection, EdaResult, Example, Probabilistic model strategy, EdaModel type

---

## Chromosome Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Solo Binary | UMDA puro: probabilidades Bernoulli por posición | |
| Binary + categórico genérico | Frecuencia por valor posible, requiere trait extra | |
| Cualquier LinearChromosome | Máxima flexibilidad — infiere modelo del tipo de gen | ✓ |

**User's choice:** Cualquier LinearChromosome
**Notes:** Flexibilidad máxima; el modelo se elige en función del bound del gene.

---

## Parent Selection

| Option | Description | Selected |
|--------|-------------|----------|
| Truncation fijo (top 50%) | Sin configuración, canónico UMDA | |
| Ratio configurable (selection_ratio) | f64 en EdaConfiguration, default 0.5 | ✓ |
| Número absoluto (num_parents) | usize directo, menos portable | |

**User's choice:** selection_ratio: f64 (default 0.5)

---

## EdaResult

| Option | Description | Selected |
|--------|-------------|----------|
| Mínimo: population, best, best_fitness, generations | Consistente con PsoResult/CmaResult | |
| Mínimo + learned_model | Añade el modelo aprendido al final | ✓ |

**User's choice:** Mínimo + learned_model

---

## Ejemplo demostrativo

| Option | Description | Selected |
|--------|-------------|----------|
| eda_onemax | Binary, maximizar sum de bits — clásico benchmark | |
| eda_trap | Función trampa deceptiva — muestra ventaja real de EDA sobre GA | ✓ |

**User's choice:** eda_trap
**Notes:** La función trampa demuestra que EDA puede aprender la estructura del problema donde GA clásico falla.

---

## Modelo de distribución (follow-up)

| Option | Description | Selected |
|--------|-------------|----------|
| Gaussiana univariada por gen | Para no-binarios: (media, desv) por posición | |
| Solo Bernoulli (binary-only en práctica) | Limita el uso real a Binary | |
| Dos estrategias según bound del gene | Bernoulli si no RealGene, Gaussiana si RealGene | ✓ |

**User's choice:** Dos estrategias según bound del gene (compile-time dispatch)

---

## Tipo de EdaModel

| Option | Description | Selected |
|--------|-------------|----------|
| Vec<f64> siempre | Sencillo, interleaved para Gaussiana | |
| Enum EdaModel { Bernoulli(Vec<f64>), Gaussian { means, stds } } | Tipo explícito, más legible | ✓ |
| Delegar al planner | Planner elige | |

**User's choice:** Enum EdaModel con variantes Bernoulli y Gaussian

---

## Claude's Discretion

- Probability clamping para Bernoulli (sugerido [0.01, 0.99])
- Std floor para Gaussian
- Mecanismo compile-time de dispatch entre modelos (dos impl blocks vs. helper trait)
- Population size default si se pasa 0 (sugerido 100)
- Serde derivation en EdaModel bajo feature flag
- GenerationStats fields

## Deferred Ideas

- Multivariate EDA (BMDA, MIMIC, BOA) — dependencias entre genes, fuera de scope UMDA
- PBIL (Population-Based Incremental Learning) — variante relacionada, posible future EdaVariant
- Adaptive selection_ratio — decaimiento con convergencia
