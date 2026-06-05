# Common Go Mistakes and Antipatterns

## Concurrency Pitfalls

### Goroutine Leaks
**Problem**: Starting goroutines without ensuring they terminate
```go
// Bad: Goroutine may never terminate
func leak() {
    ch := make(chan int)
    go func() {
        val := <-ch  // Blocks forever if nothing sends
        fmt.Println(val)
    }()
}

// Good: Use context or done channel
func noLeak(ctx context.Context) {
    ch := make(chan int)
    go func() {
        select {
        case val := <-ch:
            fmt.Println(val)
        case <-ctx.Done():
            return
        }
    }()
}
```

### Range Variable Capture in Goroutines
**Problem**: Loop variable captured by goroutine changes on each iteration
```go
// Bad: All goroutines print same (last) value
for _, val := range values {
    go func() {
        fmt.Println(val)  // Captures loop variable
    }()
}

// Good: Pass value explicitly or create local copy
for _, val := range values {
    val := val  // Create new variable
    go func() {
        fmt.Println(val)
    }()
}
```

### Missing WaitGroup or Channel Close
**Problem**: Not coordinating goroutine completion
```go
// Bad: May exit before goroutines finish
for i := 0; i < 10; i++ {
    go doWork(i)
}

// Good: Use WaitGroup
var wg sync.WaitGroup
for i := 0; i < 10; i++ {
    wg.Add(1)
    go func(i int) {
        defer wg.Done()
        doWork(i)
    }(i)
}
wg.Wait()
```

## Error Handling

### Ignoring Errors
**Problem**: Not checking error return values
```go
// Bad: Error ignored
file, _ := os.Open("config.json")

// Good: Handle errors
file, err := os.Open("config.json")
if err != nil {
    return fmt.Errorf("failed to open config: %w", err)
}
defer file.Close()
```

### Shadowing err Variable
**Problem**: Accidentally declaring new err instead of assigning
```go
// Bad: Shadows err, original error lost
err := firstOperation()
if err != nil {
    result, err := secondOperation()  // New err variable
    if err != nil {
        return err  // Returns second error, loses first
    }
}

// Good: Use assignment or different variable name
err := firstOperation()
if err != nil {
    result, err2 := secondOperation()
    if err2 != nil {
        return fmt.Errorf("first error: %v, second error: %v", err, err2)
    }
}
```

### Not Wrapping Errors
**Problem**: Losing error context
```go
// Bad: Loses context of where error occurred
if err != nil {
    return err
}

// Good: Add context with fmt.Errorf and %w
if err != nil {
    return fmt.Errorf("failed to process user %s: %w", userID, err)
}
```

## Resource Management

### Forgetting defer Close()
**Problem**: Resources not released on early returns
```go
// Bad: File not closed on error
file, err := os.Open("data.txt")
if err != nil {
    return err
}
// ... other code that might return early ...
file.Close()  // May never execute

// Good: Use defer immediately
file, err := os.Open("data.txt")
if err != nil {
    return err
}
defer file.Close()
```

### Defer in Loop
**Problem**: defer accumulates, not executed until function exits
```go
// Bad: All files stay open until function exits
for _, filename := range files {
    f, _ := os.Open(filename)
    defer f.Close()  // Doesn't close until loop finishes
    process(f)
}

// Good: Extract to separate function
for _, filename := range files {
    if err := processFile(filename); err != nil {
        return err
    }
}

func processFile(filename string) error {
    f, err := os.Open(filename)
    if err != nil {
        return err
    }
    defer f.Close()
    return process(f)
}
```

## Slices and Maps

### Slice Append Confusion
**Problem**: Not understanding slice growth and sharing
```go
// Bad: Modifying slice may affect original unexpectedly
func addElement(s []int, val int) []int {
    s = append(s, val)  // May create new backing array
    return s
}

// Good: Always use returned slice
func addElement(s []int, val int) []int {
    return append(s, val)
}
```

### Map Concurrent Access
**Problem**: Concurrent read/write to map without synchronization
```go
// Bad: Race condition
var cache map[string]string
go func() { cache["key"] = "value" }()
go func() { _ = cache["key"] }()

// Good: Use sync.RWMutex or sync.Map
var (
    cache = make(map[string]string)
    mu    sync.RWMutex
)

func set(key, val string) {
    mu.Lock()
    defer mu.Unlock()
    cache[key] = val
}

func get(key string) string {
    mu.RLock()
    defer mu.RUnlock()
    return cache[key]
}
```

### Nil Map vs Empty Map
**Problem**: Writing to nil map panics
```go
// Bad: Panics on write
var m map[string]int
m["key"] = 1  // panic: assignment to entry in nil map

// Good: Initialize with make
m := make(map[string]int)
m["key"] = 1
```

## Interface Usage

### Interface Pollution
**Problem**: Defining interfaces with too many methods or unnecessary interfaces
```go
// Bad: Over-engineered interface for simple use
type UserRepository interface {
    Create(User) error
    Update(User) error
    Delete(id int) error
    FindByID(id int) (*User, error)
    FindByEmail(email string) (*User, error)
    FindAll() ([]User, error)
    // ... many more methods
}

// Good: Accept interfaces, return structs; keep interfaces small
type UserFinder interface {
    FindByID(id int) (*User, error)
}

type UserCreator interface {
    Create(User) error
}

// Use composition when needed
type UserRepository interface {
    UserFinder
    UserCreator
}
```

### Nil Interface Gotcha
**Problem**: Interface containing nil pointer is not nil interface
```go
// Bad: Interface is not nil even though pointer is
func returnsNil() error {
    var p *MyError = nil
    return p  // Returns non-nil interface containing nil pointer
}

if err := returnsNil(); err != nil {
    // This block executes even though pointer is nil
}

// Good: Return explicit nil for interface
func returnsNil() error {
    var p *MyError = nil
    if p == nil {
        return nil
    }
    return p
}
```

## Performance

### Unnecessary String Conversions
**Problem**: Converting to string and back repeatedly
```go
// Bad: Multiple conversions
id := strconv.Atoi(strconv.Itoa(userId))

// Good: Use original type
id := userId
```

### Growing Slice Inefficiently
**Problem**: Not pre-allocating slice capacity
```go
// Bad: Multiple allocations and copies
var result []int
for i := 0; i < 1000; i++ {
    result = append(result, i)  // Grows capacity multiple times
}

// Good: Pre-allocate capacity
result := make([]int, 0, 1000)
for i := 0; i < 1000; i++ {
    result = append(result, i)
}
```

### Using + for String Concatenation in Loops
**Problem**: Creating many temporary strings
```go
// Bad: Creates many intermediate strings
var result string
for _, s := range strings {
    result += s  // Allocates new string each iteration
}

// Good: Use strings.Builder
var builder strings.Builder
for _, s := range strings {
    builder.WriteString(s)
}
result := builder.String()
```

## Testing

### Not Testing Error Cases
**Problem**: Only testing happy path
```go
// Bad: No error testing
func TestDivide(t *testing.T) {
    result := divide(10, 2)
    if result != 5 {
        t.Error("expected 5")
    }
}

// Good: Test both success and error cases
func TestDivide(t *testing.T) {
    t.Run("success", func(t *testing.T) {
        result := divide(10, 2)
        if result != 5 {
            t.Errorf("expected 5, got %v", result)
        }
    })

    t.Run("divide by zero", func(t *testing.T) {
        defer func() {
            if r := recover(); r == nil {
                t.Error("expected panic for divide by zero")
            }
        }()
        divide(10, 0)
    })
}
```

### Global State in Tests
**Problem**: Tests interfere with each other
```go
// Bad: Shared mutable state
var testDB *sql.DB

func TestA(t *testing.T) {
    testDB.Exec("INSERT INTO users ...")
}

func TestB(t *testing.T) {
    // Depends on state from TestA
}

// Good: Isolated test state
func TestA(t *testing.T) {
    db := setupTestDB(t)
    defer cleanupTestDB(t, db)
    db.Exec("INSERT INTO users ...")
}
```

## Type Assertions and Conversions

### Unchecked Type Assertion
**Problem**: Type assertion panics if wrong type
```go
// Bad: Panics if not correct type
val := i.(string)

// Good: Check with comma-ok idiom
val, ok := i.(string)
if !ok {
    return fmt.Errorf("expected string, got %T", i)
}
```

### Incorrect Nil Checks After Type Assertion
**Problem**: Checking nil after type assertion to interface
```go
// Bad: Checks wrong thing
if val, ok := err.(*MyError); ok {
    if val != nil {  // Redundant, ok is false if nil
        // ...
    }
}

// Good: ok already indicates success
if val, ok := err.(*MyError); ok {
    // val is guaranteed to be non-nil *MyError here
    fmt.Println(val.Message)
}
```

## Context Usage

### Not Propagating Context
**Problem**: Starting operations without cancellation support
```go
// Bad: Can't cancel long operation
func process(data []Item) error {
    for _, item := range data {
        if err := processItem(item); err != nil {
            return err
        }
    }
    return nil
}

// Good: Accept and check context
func process(ctx context.Context, data []Item) error {
    for _, item := range data {
        select {
        case <-ctx.Done():
            return ctx.Err()
        default:
        }
        if err := processItem(ctx, item); err != nil {
            return err
        }
    }
    return nil
}
```

### Storing Context in Struct
**Problem**: Context should be passed explicitly, not stored
```go
// Bad: Storing context in struct
type Worker struct {
    ctx context.Context
}

// Good: Pass context to methods
type Worker struct {
    // ... other fields
}

func (w *Worker) DoWork(ctx context.Context) error {
    // Use ctx parameter
}
```
