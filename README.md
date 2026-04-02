# Crack the Quote API

A RESTful API for the Crack the Quote word puzzle game. Users can solve daily puzzles or access archived puzzles by ID.

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
- `id` (integer): Puzzle ID (calculated as days since START_DATE + 1)

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

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | Required | PostgreSQL connection string |
| `START_DATE` | `2026-01-01` | Puzzle series start date (YYYY-MM-DD) |
| `PORT` | `8000` | HTTP server port |
| `DEBUG` | `false` | Enable debug logging |
| `ALLOWED_ORIGINS` | `http://localhost:3000` | CORS allowed origins (comma-separated) |

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

## Puzzle ID Calculation

Puzzle IDs are calculated based on the date offset from START_DATE:

```
puzzle_id = (puzzle_date - START_DATE).days() + 1
```

Example: If START_DATE is 2026-01-01:
- 2026-01-01 → puzzle_id = 1
- 2026-01-02 → puzzle_id = 2
- 2026-04-02 → puzzle_id = 92

---

## Caching

- **Daily puzzle**: Heavily cached, updates once per midnight UTC
- **Archive puzzles**: Fetched from database on each request

---

## License

This project is licensed under the MIT License - see the LICENSE file for details.
