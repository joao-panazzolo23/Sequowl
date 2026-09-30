tives & Behavior Guidelines for AI Agent

You must adhere strictly to the following rules when analyzing,
proposing, or editing code in this workspace.

1. Golden Rules & Directives
   Study Before Writing:

Always analyze existing project structures, architectural patterns,
design decisions, and conventions before proposing or modifying any code.

Read relevant configuration files, dependencies
(deps.edn, Cargo.toml, package.json, \*.csproj, etc.),
and existing tests to ensure full compatibility.

No Assumptions — Ask First:

If a request is ambiguous, has multiple architectural paths,
or risks breaking existing behavior, stop and ask for clarification before implementing.

Do not guess business logic or refactor untouched components without explicit instructions.

Explicit Consent for Changes:

Always detail the exact scope of changes before applying them.

Ask for confirmation before:

Adding, updating, or removing dependencies.

Modifying database schemas, public APIs, or core domain logic.

Deleting files or rewriting existing modules.

Preserve Code Style & Architecture:

Match the code style, variable naming conventions,
file structure, and architectural boundaries existing in the repository
(e.g., Clean Architecture, DDD, explicit typing, custom patterns).

Do not introduce unnecessary abstractions, extra libraries,
or design patterns unless explicitly requested or clearly required.

Quality & Compilation First:

Write fully typed, production-ready code. Do not use "Dynamic", "Any", "object", "unkown"
or anything that is not type-safe or dynamic. Prefer Generics<T>,
they're static dispatch and type safe.

Never remove strict typing, ignore compiler/linter warnings,
or introduce truncated/placeholder code (e.g., avoid // TODO: implement later unless instructed).

Ensure proposed code compiles and existing tests pass.

2. Fundamental Engineering Principles
   You must enforce Clean Code, SOLID principles, and the 4
   Pillars of Object-Oriented Programming (OOP) in all code design and modifications.

2.1. Clean Code Standards
Meaningful Names: Use clear, intention-revealing names for
variables, methods, and classes. Avoid abbreviations and generic terms.

Small & Focused Functions: Functions must be short,
perform a single logical task, and operate at a consistent level of abstraction.

Minimal Side Effects: Avoid hidden state modifications inside methods.

Readable & Self-Documenting: Write code that explains
itself without relying on excessive comments unless explaining non-obvious business domain rules.

2.2. The 4 Pillars of Object-Oriented Programming (OOP)
Abstraction: Expose only essential features and hide
implementation details behind clean interfaces or abstract bases.

Encapsulation: Restrict direct access to internal state; guard invariants using private/protected members and explicit methods or properties.

Inheritance: Model "is-a" relationships safely to share
behavior, favoring composition over inheritance where applicable.

Polymorphism: Allow derived types or interface implementations
to be treated uniformly through shared contracts without conditional type checking.

2.3. The SOLID Principles
Single Responsibility Principle (SRP): A module or class should
have one, and only one, reason to change. Keep responsibilities isolated.

Open/Closed Principle (OCP): Software entities should be open for extension,
but closed for modification. Prefer extensibility via interfaces/delegates over
modifying existing classes.

Liskov Substitution Principle (LSP): Subtypes must be substitutable for
their base types without altering correctness or breaking contract expectations.

Interface Segregation Principle (ISP): Prefer thin, client-specific
interfaces over massive, general-purpose ones. Do not force classes to
implement methods they do not use.

Dependency Inversion Principle (DIP): Depend upon abstractions, not concretions.
High-level modules should not depend on low-level modules; both should depend on abstractions.

3. Interaction & Execution Workflow
   When given a task, follow this exact process:

Context Gathering: Briefly list the files and context you inspected.

Analysis: Explain your understanding of the request and how it fits into the current codebase.

Execution Plan: Outline the steps you intend to execute.
If any step impacts critical files, dependencies, or architectural rules,
highlight it and ask for confirmation.

Implementation: Once confirmed (or if the task is straightforward and low-risk),
provide complete, precise, and fully working code modifications.

Appendix: Design Patterns Selection Guidelines
Design patterns must be applied judiciously to solve specific structural, creational, or behavioral problems. Do not force patterns into simple problems. Choose patterns based on the explicit architectural need.

1. Creational Patterns (Object Creation)
   Factory Method / Abstract Factory: Use when object creation logic is complex, dependent on runtime conditions, or needs to decouple object creation from its concrete implementation.

Builder: Use for constructing complex objects step-by-step, especially when an object requires multiple optional parameters, validation, or immutable state.

Singleton: Use only when exactly one instance of a class must exist globally (e.g., hardware drivers, low-level caches, logger infrastructure). Avoid using it as a global state container.

2. Structural Patterns (Composition & Relationships)
   Adapter: Use when integrating third-party libraries, legacy systems, or external services whose interfaces do not match the expected domain contract.

Decorator: Use to dynamically add behavior or responsibilities to an object at runtime without modifying the original class or resorting to heavy inheritance hierarchies.

Facade: Use to provide a simple, unified interface over a complex sub-system, set of APIs, or intricate framework logic.

Composite: Use when domain models represent hierarchical tree structures (e.g., UI element trees, file systems, nested expressions).

3. Behavioral Patterns (Communication & Delegation)
   Strategy: Use to encapsulate interchangeable algorithms
   or business logic variants behind a common interface,
   avoiding conditional branching (if/else or switch bloat).

Observer / Event Listener: Use for decoupled, one-to-many event
notifications when state changes in one object require updates in other independent components.

Mediator: Use to decouple direct cross-dependencies between multiple components,
routing communication through a centralized coordinator (e.g., CQRS command/query dispatchers).

State: Use when an object's behavior changes dynamically based on its internal
state machine, replacing complex state-tracking conditionals.

Chain of Responsibility: Use when a request must pass through a
sequence of processing handlers (e.g., middleware pipelines, validation pipelines,
authorization chains).

Rule of Thumb for Pattern Application
Solve real problems, not hypothetical ones:
Only introduce a pattern if the current code exhibits code smells
(e.g., duplication, rigid inheritance, massive conditionals, tight coupling)
or explicit requirements demand it.

Favor Simplicity: If a standard function, simple dependency injection,
or straightforward composition achieves the goal, prefer it over a complex pattern.

Appendix: Pragmatic Design Principles (KISS, DRY, YAGNI)
In addition to SOLID, all proposed solutions must respect fundamental software pragmatism to keep code simple, maintainable, and readable.

1. KISS (Keep It Simple, Stupid)
   Prefer Simplicity Over Cleverness: Code should be readable and obvious. Do not write complex, overly clever, or obscure code when a clear, direct solution exists.

Avoid Over-Engineering: Do not introduce unnecessary abstractions, premature microservices, indirect interfaces, or unnecessary layers unless the problem explicitly demands them.

Maintain Low Cognitive Load: Strive for clean control flow and flat structures instead of deeply nested logic, complex conditional branches, or obscure metaprogramming.

2. DRY (Don't Repeat Yourself) & Contextual Naming
   Single Source of Truth: Every piece of knowledge, business rule, or data definition must have a single, unambiguous representation within the system.

DRY in OOP Variable & Property Naming (Avoid Redundant Context):

Do Not Prefix/Suffix Property Names with Class Names: When a variable or property exists inside a class or struct, the class already provides its context. Repeating the class name in internal variables creates redundant noise and violates DRY in design.

Bad Examples (Redundant):

class User { public string UserName { get; set; } }

class Order { public decimal OrderTotalPrice { get; set; } }

class Customer { public string CustomerEmail { get; set; } }

Good Examples (Clean & DRY):

class User { public string Name { get; set; } }

class Order { public decimal TotalPrice { get; set; } }

class Customer { public string Email { get; set; } }

Rule: When referenced outside as user.Name or order.TotalPrice, the full context is clear and concise. Avoid repetition like user.UserName.

3. YAGNI (You Aren't Gonna Need It)
   Implement Only What Is Needed Now: Never build features, parameters, extra methods, or speculative flexibility for hypothetical future requirements.

Refactor When Requirements Change, Not Before: Focus exclusively on solving the immediate user prompt and current business rules.

Keep Dead Code Out: Do not leave commented-out code, unused functions, dead variables, or uncalled parameters in the repository.

In top of that, avoid using if+else statements. Prefer early returns whenever you can do it.

Appendix: Anti-Regression & Safety Guardrails
Zero Breaking Changes: Never modify existing public interface signatures, database schemas, or established API contracts unless specifically instructed to perform a breaking change.

Scope Isolation: Restrict edits exclusively to files directly related to the requested feature or fix. Do not touch adjacent files or reformat untouched code.

No Deletion Without Permission: Do not delete existing tests, utility functions, or domain models. If a piece of code appears obsolete, report it and ask for confirmation before removing it.

Fail-Fast Environment Verification: Before writing solutions, verify if the changes align with existing framework versions, SDK limitations, and environment parameters defined in project files.

Appendix: Error Handling & Logging Standards
Never Swallow Exceptions: Never leave empty catch blocks or catch generic exceptions without proper logging or handling logic.

Preserve Stack Traces: When re-throwing exceptions, ensure the original stack trace is preserved (e.g., use bare throw; in C# or wrap/chain errors explicitly without resetting the trace).

Domain-Specific Exceptions: Prefer throwing strongly-typed, domain-specific exceptions over generic runtime errors.

Structured Logging: Use structured logging principles (avoid plain string concatenation in loggers) and never log sensitive customer data (PII, tokens, passwords).

Appendix: Testing & Verification Rules
Tests Are Mandatory: When creating new business rules, domain handlers, or services, always propose corresponding unit tests.

Never Modify Passing Tests to Fix Bugs: If an existing unit test fails after a change, fix the implementation code—do not alter or weaken test assertions to pass the build, unless the requirements explicitly changed.

AAA Pattern: Structure all tests clearly using the Arrange-Act-Assert pattern.

Mock Interfaces, Not Concrete Classes: Use interface abstractions when mocking external dependencies in unit tests.

Appendix: Performance & Resource Management
Database Efficiency: Avoid N+1 query patterns. Prefer explicit batching, explicit joins, or projections over lazy-loading in loops.

Deterministic Resource Cleanup: Always dispose of or close unmanaged resources, file streams, and database connections properly using deterministic cleanup patterns (using, try-finally, or explicit disposal lifecycle).

Asynchronous & Non-Blocking: Use async/await correctly without blocking threads (avoid .Result or .Wait()). Do not create async methods without actual internal asynchronous calls.

From now on, the UI must follow a centralized, typed Design System architecture based around a single `AppTheme` object.

## Core Architecture

The application must have a global `AppTheme` property provided by `App.slint`.

`AppTheme` is the application's concrete Design System configuration.

It must contain one object for each reusable component/style type used throughout the application.

The architecture should conceptually look like this:

```text
App.slint
└── AppTheme
    ├── colors
    ├── typography
    ├── button      → ButtonStyle
    ├── card        → CardStyle
    ├── input       → InputStyle
    ├── text        → TextStyle / TypographyStyles
    └── ...
```

`AppTheme` is therefore the single source of truth for the application's visual configuration.

---

## `AppTheme`

`AppTheme` must be a concrete object containing the configured styles used by the application.

For example:

```slint
export struct AppTheme {
    colors: ThemeColors,
    typography: TypographyStyles,
    button: ButtonStyle,
    card: CardStyle,
    input: InputStyle,
}
```

The actual instance/configuration should be created and provided from `App.slint`.

Conceptually:

```slint
export component App inherits Window {
    property <AppTheme> AppTheme: {
        colors: {
            ...
        },
        typography: {
            ...
        },
        button: {
            ...
        },
        card: {
            ...
        },
        input: {
            ...
        }
    }

    // Application content
}
```

The exact initialization syntax should follow the valid Slint syntax for the current project.

The important architectural rule is that **`App.slint` owns the concrete `AppTheme` configuration**.

---

# Style Types

The individual style types must remain separate structs.

For example:

```slint
export struct ButtonStyle {
    background: color,
    foreground: color,
    border-radius: length,
    padding-horizontal: length,
    padding-vertical: length,
    text: TextStyle,
}
```

```slint
export struct CardStyle {
    background: color,
    border-radius: length,
    border-width: length,
    border-color: color,
    padding: length,
}
```

```slint
export struct InputStyle {
    background: color,
    foreground: color,
    border-color: color,
    border-radius: length,
    padding: length,
    text: TextStyle,
}
```

`AppTheme` composes these types:

```text
AppTheme
 ├── ButtonStyle
 ├── CardStyle
 ├── InputStyle
 ├── TypographyStyles
 └── ThemeColors
```

This means `AppTheme` is not itself responsible for defining every individual property. It is the **composition/root object** that brings all Design System styles together.

---

# Component Usage

A component should consume the specific style object it needs.

For example, `AppButton` uses `ButtonStyle`:

```slint
export component AppButton inherits Rectangle {
    in property <ButtonStyle> style;

    background: style.background;
    border-radius: style.border-radius;

    AppText {
        style: style.text;
    }
}
```

When used by the application:

```slint
AppButton {
    style: AppTheme.button;
    text: "Continue";
}
```

Similarly:

```slint
AppCard {
    style: AppTheme.card;
}
```

```slint
AppInput {
    style: AppTheme.input;
}
```

And typography:

```slint
AppText {
    style: AppTheme.typography.body;
}
```

The component should **not** internally access or recreate the application's concrete colors, sizes, typography, or other design decisions.

---

# Important Distinction

Do not confuse the **style type** with the **theme instance**.

For example:

```text
ButtonStyle
```

is the type.

```text
AppTheme.button
```

is the concrete instance of that type.

Likewise:

```text
CardStyle
```

is the type.

```text
AppTheme.card
```

is the concrete instance.

The relationship should always follow this pattern:

```text
AppTheme.button → ButtonStyle → AppButton
AppTheme.card   → CardStyle   → AppCard
AppTheme.input  → InputStyle  → AppInput
```

This allows the entire application's visual identity to be changed by changing `AppTheme`, without modifying the components themselves.

---

# `AppTheme` Is the Single Source of Truth

Do not create separate global style objects for individual components outside `AppTheme`.

Avoid:

```text
GlobalButtonStyle
GlobalCardStyle
GlobalInputStyle
GlobalTypography
GlobalColors
```

Instead, these should all be composed into:

```text
AppTheme
```

For example:

```slint
AppTheme.button
AppTheme.card
AppTheme.input
AppTheme.colors
AppTheme.typography
```

`AppTheme` is the root of the Design System.

---

# Component Dependency Rule

Components should receive the smallest style dependency they actually need.

Prefer:

```slint
in property <ButtonStyle> style;
```

and:

```slint
AppButton {
    style: AppTheme.button;
}
```

Do **not** make every component receive the entire `AppTheme`:

```slint
in property <AppTheme> theme;
```

unless the component genuinely needs multiple independent parts of the theme.

This keeps components decoupled from the global theme structure.

---

# No Hardcoded Design Tokens

Do not write:

```slint
background: #9C00F6;
border-radius: 8px;
font-size: 14px;
color: #FFFFFF;
```

inside reusable components when these values represent Design System decisions.

Instead:

```slint
background: style.background;
border-radius: style.border-radius;
```

and:

```slint
AppText {
    style: style.text;
}
```

The actual values belong in the `AppTheme` configuration.

---

# Design System Hierarchy

The intended architecture is:

```text
App.slint
│
└── AppTheme
    │
    ├── colors: ThemeColors
    │
    ├── typography: TypographyStyles
    │
    ├── button: ButtonStyle
    │
    ├── card: CardStyle
    │
    ├── input: InputStyle
    │
    └── other component styles...
```

And component consumption is:

```text
AppTheme
   │
   ├── button ──────→ AppButton
   │
   ├── card ────────→ AppCard
   │
   ├── input ───────→ AppInput
   │
   └── typography ──→ AppText
```

Each component receives the appropriate typed style object.

---

# Architectural Constraints

Treat the following as mandatory architectural rules:

1. `AppTheme` is the root object of the Design System.
2. `AppTheme` is configured/provided by `App.slint`.
3. Every reusable visual system should have a dedicated typed `struct`.
4. `AppTheme` contains one object for each reusable style type.
5. `AppButton` uses `ButtonStyle`.
6. `AppCard` uses `CardStyle`.
7. `AppInput` uses `InputStyle`.
8. `AppText` uses `TextStyle`.
9. Components receive style objects explicitly through properties.
10. Components must not hardcode Design System values.
11. Components should receive only the relevant style object rather than the entire `AppTheme`.
12. Do not create parallel global styling systems outside `AppTheme`.
13. Do not duplicate style values between components.
14. Reuse existing Design System structs before creating new ones.
15. If a new reusable visual concept is required, add a corresponding struct and expose it through `AppTheme`.
16. Keep the architecture simple; do not introduce abstractions merely for the sake of abstraction.

The final mental model should always be:

```text
App.slint
    ↓
AppTheme
    ↓
Typed Style Object
    ↓
UI Component
```

For example:

```text
App.slint
    ↓
AppTheme.button
    ↓
ButtonStyle
    ↓
AppButton
```

This is the Design System architecture that all new and modified UI code must follow.

## Agent Operating Rules

When working on this project, follow these rules strictly.

### File reading

You have permission to freely read and inspect project files whenever necessary to understand the codebase.

**Do NOT ask for confirmation before reading files.**

You may:

- Read any relevant source file.
- Inspect related components.
- Search the codebase.
- Inspect project configuration.
- Trace references and dependencies.
- Read multiple files when necessary to understand an implementation.
- Inspect existing Design System definitions and usages.

Do not repeatedly ask:

> "Can I read this file?"

or:

> "Would you like me to inspect this file?"

Just read the relevant files and continue.

### Changes

Before making modifications that change project files, you should present a concise summary of the intended changes and ask for confirmation.

The confirmation should happen **once for the logical change**, not once per file.

For example:

```text
I found the issue.

I will:
- Move the button styling into ButtonStyle.
- Add button: ButtonStyle to AppTheme.
- Configure AppTheme in App.slint.
- Update AppButton to consume ButtonStyle.

Proceed?
```

After confirmation, perform all necessary file modifications without asking for confirmation for every individual file.

### Do not ask redundant questions

Do not ask for confirmation when:

- Reading files.
- Searching the repository.
- Inspecting dependencies.
- Following references.
- Analyzing the existing architecture.
- Running read-only commands.
- Inspecting compiler errors.
- Inspecting project structure.

Only request confirmation before **mutating project state**.

### Group changes

Treat a coherent implementation as a single change.

If an implementation requires modifying:

```text
App.slint
AppButton.slint
Theme.slint
main.slint
```

do not request four confirmations.

Request one confirmation for the complete logical change.

### After confirmation

Once the user approves the proposed change:

- Execute the complete change.
- Do not ask for confirmation again for individual files.
- Do not stop between files waiting for permission.
- Re-read files if necessary.
- Run appropriate validation/build commands.
- Fix issues caused by the implementation when they are clearly within the approved scope.

### Read freely, write deliberately

The general rule is:

> **Reading is autonomous. Writing requires confirmation.**

You should behave as an autonomous coding agent during investigation and as a confirmation-based agent before modifying project files.

Do not create more than one class in each file. If you ever need to create more than one class, then split the code into more files.

Do not leave comments on the code. Nobody needs them and they mess up the code.

Components matter: Instead of using standard Slint components, use mostly subclasses, not superclasses.
That way, you can use multiple components and keep track of them, usefull speacially when we need to create customizable layouts and/or colors.
