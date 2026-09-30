# Sequowl — Architecture & Technology Stack

## Overview

Sequowl is a native desktop SQL client built with **Rust** and **Slint**.

The project is designed around a few core principles:

- Native desktop application
- Strong compile-time guarantees
- Clear separation of responsibilities
- Asynchronous I/O without blocking the UI
- Low runtime overhead
- Explicit dependencies between architectural layers
- Testable business logic
- Minimal coupling between the UI and application logic

The architecture follows **Clean Architecture principles**, adapted to the structure and needs of a Rust desktop application.

---

# Product Philosophy — Minimalism

Sequowl is intentionally designed as a **minimalist SQL client**.

The goal is not to build an all-in-one database management suite with every possible feature. The goal is to provide a focused, fast, native and reliable environment for working with SQL databases.

This distinction is an important product and architectural constraint.

## Minimalism as a Design Principle

Features should not be added simply because they are technically possible or because other SQL clients provide them.

Every feature introduces additional:

- UI complexity
- application state
- configuration
- cognitive overhead
- maintenance cost
- testing requirements
- architectural coupling
- potential points of failure

Therefore:

> **More functionality does not automatically mean a better SQL client.**

In some cases, additional functionality can actively make the final product worse by making common workflows harder to understand or use.

A feature that benefits a small number of advanced use cases may not justify adding permanent complexity to the experience of every user.

## Feature Discipline

Before introducing a new feature, consider:

1. Does it solve a meaningful problem for the core SQL workflow?
2. Does it make an existing workflow simpler or more capable without making it harder to understand?
3. Is the complexity introduced proportional to the value provided?
4. Can the same problem be solved with an existing feature?
5. Does the feature introduce permanent UI or architectural complexity?
6. Will users need to configure or understand it before they can use the core application?
7. Does it belong in a focused SQL client, or does it push Sequowl toward becoming a general-purpose database administration suite?

If a feature adds significant complexity without substantially improving the core experience, it should generally **not** be introduced.

## The Core Experience

Sequowl should remain focused on the fundamental workflow:

```text
Connect
   ↓
Explore
   ↓
Write SQL
   ↓
Execute
   ↓
Inspect results
```

Everything else should be evaluated against this core workflow.

The interface should remain understandable without requiring users to learn a large collection of secondary features.

## Avoid Feature Creep

Do not proactively introduce features such as:

- extensive database administration tools
- complex query builders
- excessive visualization systems
- unnecessary dashboards
- large configuration systems
- redundant database management workflows
- features added merely to match competing products
- abstractions whose primary purpose is supporting hypothetical future features

These may be valid features for other products, but they are not automatically appropriate for Sequowl.

The project should optimize for **focus rather than feature count**.

## Architectural Implication

Minimalism applies to the architecture as well as the UI.

Do not introduce:

- unnecessary layers
- unnecessary abstractions
- unnecessary dependencies
- unnecessary services
- unnecessary state
- speculative extensibility
- abstractions created only for hypothetical future requirements

Use the simplest architecture that preserves the project's required boundaries, testability and maintainability.

The architecture should be capable of growing when there is a real requirement, without forcing complexity into the application before that requirement exists.

## Decision Rule

When choosing between two valid implementations, prefer the one that:

- solves the actual requirement;
- has fewer moving parts;
- introduces less permanent complexity;
- is easier to understand;
- is easier to maintain;
- keeps the core workflow focused.

> **Sequowl should be powerful where it needs to be, not feature-rich for the sake of being feature-rich.**

The absence of a feature is not necessarily a deficiency.

In Sequowl, **simplicity is an intentional product decision**.

# Technology Stack

| Technology                  | Purpose                                |
| --------------------------- | -------------------------------------- |
| **Rust**                    | Main programming language              |
| **Slint**                   | Native declarative UI framework        |
| **Tokio**                   | Asynchronous runtime                   |
| **anyhow**                  | Application-level error handling       |
| **SQLx / database drivers** | Database connectivity                  |
| **Cargo**                   | Build system and dependency management |

The project intentionally avoids a web-based UI layer. Slint is used as the presentation technology, while Rust contains the application, domain, infrastructure, and integration logic.

---

# Why Rust?

Rust was chosen as the primary language because Sequowl is fundamentally a **native system application**.

A SQL client continuously deals with:

- Network connections
- Database drivers
- File I/O
- Multiple concurrent operations
- Connection lifecycle management
- Potentially large query results
- Background tasks
- Application state
- Resource management

Rust provides strong guarantees around these areas through its ownership and type systems.

## Main advantages

### Memory safety

Rust provides memory safety without requiring a garbage collector.

This is particularly useful for a long-running desktop application where resources such as:

- database connections
- sockets
- files
- background tasks

must have well-defined lifetimes.

### Zero-cost abstractions

The application can use abstractions such as traits, iterators, generics and domain types without necessarily introducing the runtime overhead commonly associated with dynamic abstraction layers.

### Concurrency safety

Rust's ownership model makes many classes of concurrency bugs compile-time errors.

This is especially relevant because Sequowl can have multiple independent operations running simultaneously:

```text
UI
 │
 ├── Query execution
 ├── Database metadata loading
 ├── Connection monitoring
 └── Background operations
```

### Native execution

Unlike a web-based SQL client, Sequowl does not require Chromium, WebView or a JavaScript runtime to render its interface.

The application is compiled as a native executable.

---

# Why Slint?

Slint is responsible exclusively for the **presentation layer**.

The UI is declarative, while Rust owns the application's behavior and business logic.

Conceptually:

```text
┌─────────────────────────────┐
│            Slint            │
│                             │
│  Components / Views / UI    │
└──────────────┬──────────────┘
               │
               │ events / properties
               ▼
┌─────────────────────────────┐
│            Rust             │
│                             │
│ Application / Domain / Infra│
└─────────────────────────────┘
```

This separation prevents the UI framework from becoming the center of the application's architecture.

Slint should describe **what the interface looks like**, while Rust determines **what the application does**.

---

# Why Tokio?

Sequowl performs operations that are naturally asynchronous:

- Opening database connections
- Executing queries
- Reading query results
- Loading database metadata
- Network communication
- File operations
- Background tasks

These operations should not block the UI thread.

Tokio provides the asynchronous runtime responsible for executing these operations.

For example:

```text
Slint UI
   │
   │ user executes query
   ▼
Application
   │
   │ spawn async operation
   ▼
Tokio Runtime
   │
   ├── Database I/O
   ├── Network I/O
   └── Background tasks
   │
   ▼
Application
   │
   │ update state
   ▼
Slint UI
```

The important architectural rule is:

> **The UI must never be responsible for executing blocking I/O directly.**

Tokio provides the execution environment, while the application layer decides **what asynchronous operation should happen**.

---

# Error Handling with `anyhow`

The project uses `anyhow::Result` primarily at the **application boundary**.

Instead of manually propagating every infrastructure error type through the entire application, application-level operations can use:

```rust
use anyhow::Result;

async fn execute_query(...) -> Result<QueryResult> {
    // ...
}
```

This is useful for operations where the caller primarily needs to know:

> Did the operation succeed, and if not, what went wrong?

`anyhow` also provides contextual error information:

```rust
use anyhow::{Context, Result};

async fn connect(...) -> Result<Connection> {
    create_connection()
        .await
        .context("failed to connect to database")
}
```

This produces errors that are much more useful when displayed or logged.

## Where `anyhow` belongs

`anyhow` is intended primarily for **application-level error propagation**.

Domain logic should not depend on `anyhow` when a structured domain error can express the failure more precisely.

For example:

```text
Domain
  │
  └── DomainError

Infrastructure
  │
  └── Driver / I/O errors

Application
  │
  └── anyhow::Result
```

This keeps the domain independent from infrastructure and application-specific error handling.

---

# Clean Architecture

Sequowl follows Clean Architecture principles through its directory structure.

The objective is to control dependencies rather than simply divide the project into folders.

The fundamental rule is:

> **Dependencies point inward.**

The domain should not depend on:

- Slint
- Tokio
- SQL drivers
- filesystem implementations
- UI components
- infrastructure details

Instead, outer layers depend on abstractions defined by inner layers.

---

# Project Structure

The project is organized approximately as follows:

```text
src/
├── core/
│   ├── entities/
│   ├── value_objects/
│   ├── repositories/
│   └── errors/
│
├── application/
│   ├── use_cases/
│   ├── services/
│   ├── dto/
│   └── errors/
│
├── infrastructure/
│   ├── database/
│   ├── persistence/
│   ├── repositories/
│   └── configuration/
│
├── presentation/
│   ├── components/
│   ├── pages/
│   ├── view_models/
│   └── ui/
│
├── shared/
│   ├── ...
│
└── main.rs
```

The exact organization may evolve as the project grows, but the architectural boundaries should remain consistent.

## Design System

Sequowl uses a centralized design system built with **Slint**, designed to provide consistent visual language, reusable component styles, and runtime theme customization while keeping UI components decoupled from the global theme configuration.

The design system is organized around two complementary concepts: **design tokens** and **component styles**.

### Design Tokens

Primitive and semantic values are centralized into reusable token structures:

- `AppColors` — semantic colors such as primary, background, surface, text, and borders.
- `AppSpacings` — standardized spacing values.
- `AppRadius` — standardized corner radii.
- `AppTypography` — reusable typography definitions.

These tokens represent the visual foundation of the application and should not contain component-specific behavior.

### Component Styles

Component-specific visual configurations are built from the design tokens.

For example, buttons expose reusable `ButtonStyle` definitions such as:

- `primary`
- `secondary`
- `outlined`
- `ghost`

The same approach is used for other components such as cards, inputs, and future UI primitives.

This creates a clear separation between **what a design token represents** and **how a component uses that token**.

```text
Design Tokens
    │
    ├── Colors
    ├── Spacing
    ├── Radius
    └── Typography
          │
          ▼
Component Styles
    │
    ├── ButtonStyles
    ├── CardStyles
    └── InputStyles
          │
          ▼
UI Components
    │
    ├── AppButton
    ├── AppCard
    └── AppInput
```

### Global Theme

Slint does not provide an object-style styling mechanism equivalent to Flutter's `ThemeData`, where an entire style hierarchy can be implicitly propagated through the widget tree.

Instead, Sequowl uses a global `Theme` object containing the complete `AppTheme` configuration.

Conceptually:

```text
Theme
│
├── colors
├── typography
├── spacing
├── radius
│
└── component styles
    ├── buttons
    ├── cards
    └── inputs
```

Components do not need to receive the entire `AppTheme`. Instead, components expose properties for the specific style they require:

```slint
AppButton {
    style: Theme.value.buttons.primary;
}
```

This prevents components from becoming coupled to the complete theme configuration and keeps their dependencies explicit.

### Runtime Theming

The global theme is also the boundary between the Rust backend and the Slint UI.

Theme configuration can be constructed or modified by Rust at runtime and assigned to the global `Theme` object. Since Slint properties are reactive, components bound to theme values automatically update when the theme changes.

```text
Backend / Configuration
        │
        ▼
    AppTheme
        │
        ▼
   Theme.value
        │
        ▼
 Reactive Slint Bindings
        │
        ▼
         UI
```

This allows Sequowl to support different visual themes, including light/dark themes and potentially user-defined themes, without coupling backend logic to individual UI components.

### Design Principles

The design system follows these principles:

- **Centralized visual language** — visual decisions are defined in one place.
- **Semantic tokens** — colors, spacing, typography, and radii describe their purpose rather than individual components.
- **Composable styles** — component styles are composed from reusable tokens.
- **Explicit dependencies** — components depend only on the styles or values they actually require.
- **Runtime customization** — theme values can be changed without rebuilding the UI.
- **Backend/UI separation** — Rust provides theme configuration, while Slint owns its visual representation.
- **Minimal boilerplate** — global theme state avoids manually passing the entire theme through every component.
- **Reusable components** — application components consume standardized styles instead of defining their own visual language.

The goal is to make the UI **consistent by default, customizable when necessary, and structurally independent from the source of the theme configuration**.

---

# 1. Domain

The domain is the innermost layer.

It contains the concepts that represent the actual problem being solved.

Examples:

```text
Database
Connection
Table
Column
Query
QueryResult
DatabaseProvider
```

The domain should contain business rules and abstractions that are independent of external technologies.

For example:

```rust
pub trait DatabaseRepository {
    async fn connect(&self, connection: &Connection) -> Result<...>;
}
```

The domain defines **what is required**, not **how it is implemented**.

The domain should not know that:

- PostgreSQL is implemented using a specific driver
- SQLite uses a particular library
- the UI is implemented using Slint
- asynchronous execution is provided by Tokio

---

# 2. Application

The application layer coordinates use cases.

It answers questions such as:

- What happens when a database is connected?
- What happens when a query is executed?
- How is database metadata loaded?
- How is a connection removed
