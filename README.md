# Crack the Quote API

A RESTful API for the Crack the Quote word puzzle game. Users can solve daily puzzles or access archived puzzles by ID.

Authentication is handled via **server-managed OAuth2 with secure HttpOnly cookies**, ensuring tokens are never exposed to JavaScript.

---

## Authentication

### Overview

This API uses **Google OAuth2 Authorization Code Flow + PKCE** for authentication:

1. Frontend calls `GET /auth/google/url` to get the authorization URL
2. User is redirected to Google's login page
3. After login, Google redirects back to the frontend with an authorization code
4. Frontend calls `GET /auth/google/callback?code=...&state=...&code_verifier=...`
5. Backend exchanges the code for an ID token, verifies it, and issues an `auth_token` cookie
6. All subsequent requests include this cookie automatically

The `auth_token` is a **JWT signed with the server's secret**, stored in an **HttpOnly, Secure cookie** that JavaScript cannot access.

### Authentication Endpoints

#### `GET /auth/google/url`

Initiates OAuth2 flow by returning the authorization URL and PKCE parameters.

**Response (200 OK):**
```json
{
  "url": "https://accounts.google.com/o/oauth2/v2/auth?...",
  "state": "<random-uuid>",
  "code_verifier": "<128-char-pkce-verifier>"
}
```

**Frontend responsibilities:**
- Store `state` and `code_verifier` in `sessionStorage`
- Redirect user to the `url`
- After redirect back, retrieve these values and send them to `/auth/google/callback`

---

#### `GET /auth/google/callback`

OAuth2 callback handler. Processes authorization response from Google, validates PKCE, and issues session cookie.

**Query Parameters:**
- `code`: Authorization code from Google
- `state`: CSRF token (must match the value from `/auth/google/url`)
- `code_verifier`: PKCE verifier (must match the code challenge sent to Google)

**Response (200 OK):**
```json
{
  "userId": "<uuid>",
  "email": "user@example.com"
}
```

**Cookie Set:**
- `auth_token`: JWT session token
  - HttpOnly: true (not readable by JavaScript)
  - Secure: true (only sent over HTTPS in production)
  - SameSite: Lax
  - Path: /
  - Max-Age: 86400 seconds (24 hours)

**Errors:**
- `400 Bad Request`: Missing or invalid query parameters
- `500 Internal Server Error`: OAuth provider error or token validation failure

---

#### `POST /auth/logout`

Clears the session by removing the auth_token cookie.

**Response (200 OK):**
```json
{
  "message": "Logged out"
}
```

---

## API Endpoints

### Daily Puzzle

#### `GET /puzzles/daily`
Fetch today's puzzle (heavily cached, updates daily)

**Response (200 OK):**
```json
{
  "id": 100,
  "encoded_quote": "gu g ws ltz dwfo bu ltz xbej",
  "author": "Megan Whalen Turner",
  "source": "The Queen of Attolia",
  "date": "2026-04-02"
}
```

*Note: `source` field may be `null` if not set in the quotes table.*

**Error (503 Service Unavailable):**
If puzzle hasn't been generated yet:
```json
{
  "title": "Puzzle not ready",
  "status": 503,
  "detail": "Please wait while puzzle is being generated.",
  "instance": null
}
```

---

#### `POST /puzzles/daily/check-letter`
Check if a cipher letter maps to a guessed letter

**Request:**
```json
{
  "letterToCheck": "a",
  "cipherLetter": "g"
}
```

**Response (200 OK):**
```json
{
  "isLetterCorrect": true
}
```

---

#### `POST /puzzles/daily/solve-letter`
Reveal the correct letter for a cipher letter

**Request:**
```json
{
  "cipherLetter": "g"
}
```

**Response (200 OK):**
```json
{
  "correctLetter": "a"
}
```

---

#### `POST /puzzles/daily/check-quote`
Validate the entire cipher map solution

**Request:**
```json
{
  "cipherMap": {
    "g": "a",
    "u": "b",
    "w": "c",
    ...
  }
}
```

**Response (200 OK):**
```json
{
  "isQuoteCorrect": true
}
```

---

### Archive Puzzles

#### `GET /puzzles/{id}`
Fetch a specific puzzle by ID

**Parameters:**
- `id` (uuid): Puzzle ID

**Response (200 OK):**
Same as daily puzzle response

**Error (404 Not Found):**
If puzzle ID corresponds to a future date or puzzle doesn't exist:
```json
{
  "title": "Invalid request",
  "status": 404,
  "detail": "Resource not found",
  "instance": null
}
```

---

#### `POST /puzzles/{id}/check-letter`
#### `POST /puzzles/{id}/solve-letter`
#### `POST /puzzles/{id}/check-quote`

Same request/response format as daily endpoints, but for archived puzzles.

---

### Health Check

#### `GET /health`
Verify API is running

**Response (200 OK):**
```json
{
  "status": "healthy"
}
```

---

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `GOOGLE_CLIENT_ID` | Yes | — | OAuth client ID from [Google Console](https://console.cloud.google.com) |
| `GOOGLE_CLIENT_SECRET` | Yes | — | OAuth client secret from Google Console |
| `JWT_SECRET` | Yes | — | Secret key for signing session JWTs (use a random 32+ character string) |
| `DEBUG` | No | `false` | Enable debug logging |
| `ALLOWED_ORIGINS` | No | `http://localhost:3000` | CORS allowed origins (comma-separated) |
| `GOOGLE_REDIRECT_URI` | No | `http://localhost:3000/auth/callback` | OAuth redirect URI (must match Google Console exactly) |
| `SECURE_COOKIES` | No | `true` | Use Secure flag for cookies (set to `false` only for local dev without HTTPS) |

---

## Error Handling

All errors follow RFC 7807 Problem Details format:

```json
{
  "title": "Error type",
  "status": 400,
  "detail": "Human-readable description",
  "instance": null
}
```

| Status | Scenario |
|--------|----------|
| 400 | Validation error (bad input) |
| 404 | Resource not found / puzzle in future |
| 500 | Internal server error |
| 503 | Puzzle not generated yet |

---

## Caching

- **Daily puzzle**: Heavily cached, updates once per midnight UTC
- **Archive puzzles**: Fetched from database on each request

---

## License

This project is licensed under the MIT License - see the LICENSE file for details.
