# SuccessFactors Employee Central Microservice

A Rust microservice for reading SAP SuccessFactors Employee Central data via the OData API.

## Features

- RESTful API endpoints for accessing Employee Central data
- Support for multiple SuccessFactors entities:
  - Users
  - Personal Information (PerPersonal)
  - Job Information (EmpJob)
  - Employment Information (EmpEmployment)
  - Email Addresses (PerEmail)
  - Phone Numbers (PerPhone)
- Pagination support with `top` and `skip` parameters
- Health check and connection test endpoints
- CORS support for web clients
- Structured logging with tracing

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- SAP SuccessFactors account with API access
- Valid API credentials (username, password, company ID)

## Configuration

Copy the example environment file and configure your credentials:

```bash
cp .env.example .env
```

Edit `.env` with your SuccessFactors credentials:

```env
SF_API_URL=https://api.successfactors.com
SF_COMPANY_ID=your_company_id
SF_USERNAME=your_username
SF_PASSWORD=your_password
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
```

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

The server will start on the configured host and port (default: `http://0.0.0.0:3000`).

## API Endpoints

### Health & Status

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/api/test-connection` | GET | Test SuccessFactors API connection |

### Users

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/users` | GET | Get all users |
| `/api/users/:user_id` | GET | Get a specific user by ID |

### Employee Central Data

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/employees/personal` | GET | Get personal information |
| `/api/employees/jobs` | GET | Get job information |
| `/api/employees/employment` | GET | Get employment information |
| `/api/employees/emails` | GET | Get email addresses |
| `/api/employees/phones` | GET | Get phone numbers |

### Pagination

All list endpoints support pagination via query parameters:

- `top`: Number of records to return (e.g., `?top=10`)
- `skip`: Number of records to skip (e.g., `?skip=20`)

Example: `/api/users?top=10&skip=0`

## Example Requests

```bash
# Health check
curl http://localhost:3000/health

# Test connection
curl http://localhost:3000/api/test-connection

# Get first 10 users
curl "http://localhost:3000/api/users?top=10"

# Get specific user
curl http://localhost:3000/api/users/jsmith

# Get employee personal info with pagination
curl "http://localhost:3000/api/employees/personal?top=20&skip=0"
```

## Project Structure

```
├── Cargo.toml           # Project dependencies
├── .env.example         # Example environment configuration
├── .gitignore           # Git ignore rules
└── src/
    ├── main.rs          # Application entry point
    ├── config.rs        # Configuration management
    ├── client.rs        # SuccessFactors OData API client
    ├── models.rs        # Data models and DTOs
    └── handlers.rs      # HTTP request handlers
```

## License

MIT
