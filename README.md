# notes-api

Rust API for user authentication and private notes.

## Requirements

- Rust
- Docker and Docker Compose
- `sqlx-cli`

Install `sqlx-cli` if needed:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

## Environment

Copy the example file:

```bash
cp .env.example .env
```

Important URL difference:

- Inside Docker, the database host is `postgres`.
- From your host terminal, the database host is `localhost`.

The repo also sets local Cargo env values in `.cargo/config.toml` for development and tests.

## Run Locally

Start Postgres and Redis:

```bash
docker compose up -d postgres redis
```

Run migrations:

```bash
DATABASE_URL=postgres://postgres:1234@localhost:5432/app_db cargo sqlx migrate run
```

Start the API:

```bash
cargo run --bin notes_api
```

The API runs on:

```text
http://localhost:3000
```

Check health:

```bash
curl http://localhost:3000/health
```

Expected response:

```json
{"status":"ok"}
```

## Run Tests

The tests use `app_db_test`.

Create it if it does not exist:

```bash
docker exec notes-api-postgres createdb -U postgres app_db_test
```

Run migrations on the test database:

```bash
DATABASE_URL=postgres://postgres:1234@localhost:5432/app_db_test cargo sqlx migrate run
```

Run tests:

```bash
cargo test -- --nocapture
```

Tests run with one thread because DB tests truncate shared tables.

## API Examples

Register:

```bash
curl -i -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com","password":"secret123"}'
```

Login:

```bash
curl -i -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"secret123"}'
```

The auth response contains access and refresh tokens:

```json
{
  "access_token": "...",
  "refresh_token": "...",
  "token": "Bearer"
}
```

Use it on protected routes:

```bash
TOKEN="paste_access_token_here"
```

Refresh the access token:

```bash
curl -i -X POST http://localhost:3000/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token":"paste_refresh_token_here"}'
```

The refresh response returns a new access token and a new refresh token. The old refresh token is revoked after use.

Logout:

```bash
curl -i -X POST http://localhost:3000/auth/logout \
  -H "Content-Type: application/json" \
  -d '{"refresh_token":"paste_refresh_token_here"}'
```

Create a note:

```bash
curl -i -X POST http://localhost:3000/notes \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"First note","content":"Hello"}'
```

List your notes:

```bash
curl -i http://localhost:3000/notes \
  -H "Authorization: Bearer $TOKEN"
```

Get one note:

```bash
curl -i http://localhost:3000/notes/1 \
  -H "Authorization: Bearer $TOKEN"
```

Update a note:

```bash
curl -i -X PUT http://localhost:3000/notes/1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"title":"Updated title","content":"Updated content"}'
```

Delete a note:

```bash
curl -i -X DELETE http://localhost:3000/notes/1 \
  -H "Authorization: Bearer $TOKEN"
```

## Current Routes

Public:

```text
GET  /health
POST /auth/register
POST /auth/login
POST /auth/refresh
POST /auth/logout
```

Protected:

```text
GET    /users
GET    /users/{id}
PUT    /users/{id}
DELETE /users/{id}
GET    /notes
POST   /notes
GET    /notes/{id}
PUT    /notes/{id}
DELETE /notes/{id}
```

Notes are scoped to the authenticated user. A user cannot read, update, or delete another user's notes.
