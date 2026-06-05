# Effective Go: Key Best Practices

## Formatting
- Use `gofmt` for automatic code formatting to maintain consistency across projects
- Employ tabs for indentation; spaces are discouraged
- "Go has no line length limit" but wrap long lines and indent with extra tabs for readability
- Control structures require mandatory braces, even for single-statement bodies

## Commentary
Line comments (`//`) are standard; block comments (`/* */`) typically appear as package documentation. Doc comments preceding declarations without intervening blank lines document that declaration and serve as primary package documentation.

## Naming Conventions
- **Packages**: Use lowercase, single-word names; avoid underscores and mixed case
- **Exported names**: Begin with uppercase; unexported names start lowercase
- **Getters**: Omit "Get" prefix (use `Owner()` not `GetOwner()`)
- **Interfaces**: One-method interfaces use method name plus "-er" suffix (`Reader`, `Writer`)
- **Multiword names**: Use `MixedCaps` rather than underscores

## Control Structures
- Omit unnecessary `else` statements when preceding blocks end in `return`, `break`, or `continue`
- Initialize variables within `if`/`switch` statements using the initialization clause
- Use type switches to discover interface dynamic types
- Label loops when `break` must exit outer loop rather than switch

## Functions
- Leverage multiple return values, particularly for returning results with error indicators
- Name result parameters to clarify which returned value is which and enable unadorned `return` statements
- Use `defer` to schedule cleanup operations (file closing, mutex unlocking) guaranteeing execution

## Data Structures

**Arrays**: Pass pointers to avoid expensive full copies; generally prefer slices instead.

**Slices**: Wrap arrays for convenient sequence operations; modifications affect underlying array.

**Maps**: Keys require equality operators; slices cannot be map keys. The "comma ok" idiom tests presence: `value, ok := m[key]`

**Composite literals**: Allocate and initialize structures in single expressions; field labels enable any order.

## Interfaces and Types
- Interfaces define behavior requirements without enforcing implementation contracts
- Embed types to inherit methods; embedded type methods have embedded type as receiver, not outer type
- Convert types to access different method sets; conversions don't create new values (except int-to-float conversions)
- Use blank identifier (`_`) to verify compile-time interface satisfaction

## Error Handling
- Return errors alongside normal values using multiple returns
- Implement custom error types wrapping system errors with contextual information
- Use type assertions to extract specific error details: `if e, ok := err.(*PathError); ok {}`
- Reserve `panic` for unrecoverable initialization failures; return errors during normal operation

## Concurrency
- Apply the principle: "Do not communicate by sharing memory; instead, share memory by communicating"
- Launch goroutines with `go` keyword; they're lightweight and multiplexed across OS threads
- Use unbuffered channels for synchronization; buffered channels act as semaphores
- Implement channels of channels for safe demultiplexing in concurrent systems
- Use `defer` with `recover()` inside goroutines to prevent one failure from crashing others
