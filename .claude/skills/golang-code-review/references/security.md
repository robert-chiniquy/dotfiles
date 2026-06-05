# Go Security Best Practices

## Input Validation and Sanitization

### SQL Injection Prevention
**Problem**: Concatenating user input into SQL queries
```go
// Bad: SQL injection vulnerability
query := "SELECT * FROM users WHERE email = '" + userEmail + "'"
db.Query(query)

// Good: Use parameterized queries
query := "SELECT * FROM users WHERE email = ?"
db.Query(query, userEmail)

// Or with named parameters (PostgreSQL)
query := "SELECT * FROM users WHERE email = $1"
db.Query(query, userEmail)
```

### Command Injection Prevention
**Problem**: Passing unsanitized user input to shell commands
```go
// Bad: Command injection vulnerability
cmd := exec.Command("sh", "-c", "ls "+userDir)

// Good: Use argument list instead of shell
cmd := exec.Command("ls", userDir)

// Better: Validate and sanitize user input first
func sanitizePath(path string) (string, error) {
    clean := filepath.Clean(path)
    if strings.Contains(clean, "..") {
        return "", errors.New("invalid path")
    }
    return clean, nil
}
```

### Path Traversal Prevention
**Problem**: Not validating file paths from user input
```go
// Bad: Path traversal vulnerability
func serveFile(w http.ResponseWriter, r *http.Request) {
    filename := r.URL.Query().Get("file")
    http.ServeFile(w, r, "/var/www/"+filename)  // Can access ../../../../etc/passwd
}

// Good: Validate and restrict to allowed directory
func serveFile(w http.ResponseWriter, r *http.Request) {
    filename := filepath.Base(r.URL.Query().Get("file"))  // Remove directory components
    fullPath := filepath.Join("/var/www/", filename)

    // Ensure path is within allowed directory
    if !strings.HasPrefix(filepath.Clean(fullPath), "/var/www/") {
        http.Error(w, "Invalid file path", http.StatusBadRequest)
        return
    }

    http.ServeFile(w, r, fullPath)
}
```

### HTML/JavaScript Injection (XSS)
**Problem**: Outputting unsanitized user input in HTML
```go
// Bad: XSS vulnerability
fmt.Fprintf(w, "<div>%s</div>", userInput)

// Good: Use html/template which auto-escapes
tmpl := template.Must(template.New("page").Parse("<div>{{.}}</div>"))
tmpl.Execute(w, userInput)
```

## Authentication and Authorization

### Insecure Password Storage
**Problem**: Storing passwords in plaintext or with weak hashing
```go
// Bad: Plaintext password storage
user.Password = password

// Bad: Weak hashing (MD5, SHA1)
hash := md5.Sum([]byte(password))

// Good: Use bcrypt
import "golang.org/x/crypto/bcrypt"

hashedPassword, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
if err != nil {
    return err
}

// Verify password
err = bcrypt.CompareHashAndPassword(hashedPassword, []byte(password))
```

### Session Token Security
**Problem**: Predictable or insecure session tokens
```go
// Bad: Predictable token
token := fmt.Sprintf("%d", time.Now().Unix())

// Good: Cryptographically secure random token
import "crypto/rand"

func generateToken() (string, error) {
    b := make([]byte, 32)
    if _, err := rand.Read(b); err != nil {
        return "", err
    }
    return base64.URLEncoding.EncodeToString(b), nil
}
```

### JWT Security
**Problem**: Not validating JWT properly or using weak algorithms
```go
// Bad: Accepting "none" algorithm
token, err := jwt.Parse(tokenString, func(token *jwt.Token) (interface{}, error) {
    return []byte(secretKey), nil  // No algorithm check
})

// Good: Validate algorithm and signing method
token, err := jwt.Parse(tokenString, func(token *jwt.Token) (interface{}, error) {
    // Validate algorithm
    if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
        return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
    }
    return []byte(secretKey), nil
})
```

### Insufficient Authorization Checks
**Problem**: Not verifying user permissions before operations
```go
// Bad: No authorization check
func deleteUser(userID int) error {
    return db.Delete("users", userID)
}

// Good: Check authorization
func deleteUser(ctx context.Context, userID int) error {
    currentUser := getUserFromContext(ctx)

    // Verify current user has permission to delete
    if !currentUser.IsAdmin() && currentUser.ID != userID {
        return errors.New("unauthorized")
    }

    return db.Delete("users", userID)
}
```

## Cryptography

### Weak Random Number Generation
**Problem**: Using math/rand for security-sensitive operations
```go
// Bad: Predictable random numbers
import "math/rand"
sessionID := rand.Int()

// Good: Use crypto/rand for security
import "crypto/rand"
import "math/big"

func randomInt() (int, error) {
    n, err := rand.Int(rand.Reader, big.NewInt(1000000))
    if err != nil {
        return 0, err
    }
    return int(n.Int64()), nil
}
```

### Hardcoded Secrets
**Problem**: Embedding secrets in source code
```go
// Bad: Hardcoded credentials
const apiKey = "sk-1234567890abcdef"
const dbPassword = "super_secret_password"

// Good: Use environment variables or secret management
apiKey := os.Getenv("API_KEY")
if apiKey == "" {
    return errors.New("API_KEY not set")
}
```

### Insecure TLS Configuration
**Problem**: Disabling certificate verification or using weak TLS versions
```go
// Bad: Disables certificate verification
client := &http.Client{
    Transport: &http.Transport{
        TLSClientConfig: &tls.Config{
            InsecureSkipVerify: true,  // Never do this in production
        },
    },
}

// Good: Proper TLS configuration
client := &http.Client{
    Transport: &http.Transport{
        TLSClientConfig: &tls.Config{
            MinVersion: tls.VersionTLS12,  // Minimum TLS 1.2
            CipherSuites: []uint16{
                tls.TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                tls.TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
            },
        },
    },
}
```

### Weak Encryption
**Problem**: Using deprecated or weak encryption algorithms
```go
// Bad: DES is weak and deprecated
import "crypto/des"

// Good: Use AES with GCM mode
import "crypto/aes"
import "crypto/cipher"

func encrypt(plaintext []byte, key []byte) ([]byte, error) {
    block, err := aes.NewCipher(key)
    if err != nil {
        return nil, err
    }

    gcm, err := cipher.NewGCM(block)
    if err != nil {
        return nil, err
    }

    nonce := make([]byte, gcm.NonceSize())
    if _, err := rand.Read(nonce); err != nil {
        return nil, err
    }

    ciphertext := gcm.Seal(nonce, nonce, plaintext, nil)
    return ciphertext, nil
}
```

## HTTP Security

### Missing Security Headers
**Problem**: Not setting security-related HTTP headers
```go
// Bad: No security headers
http.HandleFunc("/", handler)

// Good: Add security headers middleware
func securityHeaders(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        w.Header().Set("X-Content-Type-Options", "nosniff")
        w.Header().Set("X-Frame-Options", "DENY")
        w.Header().Set("X-XSS-Protection", "1; mode=block")
        w.Header().Set("Strict-Transport-Security", "max-age=31536000; includeSubDomains")
        w.Header().Set("Content-Security-Policy", "default-src 'self'")
        next.ServeHTTP(w, r)
    })
}
```

### CORS Misconfiguration
**Problem**: Overly permissive CORS settings
```go
// Bad: Allows all origins
w.Header().Set("Access-Control-Allow-Origin", "*")
w.Header().Set("Access-Control-Allow-Credentials", "true")  // Dangerous with *

// Good: Whitelist specific origins
allowedOrigins := map[string]bool{
    "https://example.com": true,
    "https://app.example.com": true,
}

origin := r.Header.Get("Origin")
if allowedOrigins[origin] {
    w.Header().Set("Access-Control-Allow-Origin", origin)
    w.Header().Set("Access-Control-Allow-Credentials", "true")
}
```

### Unvalidated Redirects
**Problem**: Redirecting to user-supplied URLs without validation
```go
// Bad: Open redirect vulnerability
func redirect(w http.ResponseWriter, r *http.Request) {
    target := r.URL.Query().Get("url")
    http.Redirect(w, r, target, http.StatusFound)  // Can redirect anywhere
}

// Good: Validate redirect URL
func redirect(w http.ResponseWriter, r *http.Request) {
    target := r.URL.Query().Get("url")

    // Parse and validate URL
    targetURL, err := url.Parse(target)
    if err != nil || !isAllowedHost(targetURL.Host) {
        http.Error(w, "Invalid redirect", http.StatusBadRequest)
        return
    }

    http.Redirect(w, r, target, http.StatusFound)
}

func isAllowedHost(host string) bool {
    allowed := []string{"example.com", "www.example.com"}
    for _, h := range allowed {
        if host == h {
            return true
        }
    }
    return false
}
```

## Rate Limiting and DoS Prevention

### Missing Rate Limiting
**Problem**: No protection against request flooding
```go
// Bad: No rate limiting
http.HandleFunc("/api/login", loginHandler)

// Good: Implement rate limiting
import "golang.org/x/time/rate"

var loginLimiter = rate.NewLimiter(1, 5)  // 1 req/sec, burst of 5

func loginHandler(w http.ResponseWriter, r *http.Request) {
    if !loginLimiter.Allow() {
        http.Error(w, "Too many requests", http.StatusTooManyRequests)
        return
    }
    // ... handle login
}
```

### Resource Exhaustion
**Problem**: Not limiting resource consumption
```go
// Bad: Unlimited read
body, _ := ioutil.ReadAll(r.Body)

// Good: Limit read size
const maxBodySize = 1 << 20  // 1 MB
body, err := ioutil.ReadAll(io.LimitReader(r.Body, maxBodySize))
if err != nil {
    http.Error(w, "Request too large", http.StatusRequestEntityTooLarge)
    return
}
```

## Logging and Error Handling

### Exposing Sensitive Information in Errors
**Problem**: Returning detailed errors to users
```go
// Bad: Exposes database structure
if err != nil {
    http.Error(w, err.Error(), http.StatusInternalServerError)  // May expose "table users column password_hash"
}

// Good: Log detailed error, return generic message
if err != nil {
    log.Printf("database error: %v", err)  // Detailed error in logs
    http.Error(w, "Internal server error", http.StatusInternalServerError)  // Generic error to user
}
```

### Logging Sensitive Data
**Problem**: Including passwords, tokens, or PII in logs
```go
// Bad: Logs password
log.Printf("User login: email=%s password=%s", email, password)

// Good: Don't log sensitive data
log.Printf("User login attempt: email=%s", email)
```

## Dependency Management

### Using Vulnerable Dependencies
Check for known vulnerabilities:
```bash
# Run regularly
go list -json -m all | nancy sleuth
govulncheck ./...
```

### Not Pinning Dependency Versions
```go
// Bad: In go.mod, using indirect dependencies without version control
require github.com/some/package latest

// Good: Use specific versions and go.sum for integrity
require github.com/some/package v1.2.3
```
