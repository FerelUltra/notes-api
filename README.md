# notes-api

## Run with Docker

Create `.env`:

```env
DATABASE_URL=postgres://postgres:1234@postgres:5432/app_db
RUST_LOG=notes_api=debug, tower_http=debug