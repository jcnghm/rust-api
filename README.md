<div align="center">
  <img src="https://raw.githubusercontent.com/rust-lang/rust-artwork/master/logo/rust-logo-256x256.png" alt="Rust Logo" width="100">
  
# Actix Web Rust API

A REST API framework built with Rust and Actix Web, featuring JWT authentication, and CRUD operations with SQLite database support.
</div>

## Features

**Authentication & Authorization**

- JWT-based authentication with custom middleware
- Role-based access control (admin/user roles)
- Secure token validation

**Database Support**

- SQLite database with SQLx ORM
- Migrations, connection pooling

**API Capabilities**

- RESTful CRUD operations for objects
- Health check endpoint
- Request/response logging
- Structured error handling

**Architecture**

- Handlers, repositories, services
- Auth Middleware

## Quick Start

### Prerequisites

- Rust 1.70+ and Cargo
- SQLite
### Installation

```bash
# Clone the repository
cd rust-api-framework

# Create data directory for SQLite database
mkdir -p data

# Install dependencies and run
cargo run
```

The server will start at `http://127.0.0.1:8080` by default with SQLite database at `./data/app.db`.

## Database Setup

The application automatically creates and migrates the SQLite database on startup. The database file is created at `./data/app.db` by default.

### Manual Database Setup

If you want to manually initialize the database with sample data:

```bash
# Create the database file
touch data/app.db

# Run the initialization script
sqlite3 data/app.db < scripts/init_db.sql
```

### Database Configuration

The database URL can be configured via environment variables:

```env
DATABASE_URL=sqlite:./data/app.db
```

## API Endpoints

### Authentication

- `POST /token` - Login with username/password

### Protected Endpoints

All endpoints below require `Authorization: Bearer <token>` header:

- `GET /health` - Health check
- `GET /objects` - List all objects
- `GET /objects/{id}` - Get specific object
- `POST /objects` - Create new object
- `PUT /objects/{id}` - Update object
- `PATCH /objects/{id}` - Partial update
- `DELETE /objects/{id}` - Delete object

## Authentication

### JWT Authentication

```bash
# Login to get token
curl -X POST http://localhost:8080/token \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password123"}'

# Use token in requests
curl -H "Authorization: Bearer <your-token>" \
  http://localhost:8080/objects
```

### Demo Credentials

- **Admin**: `admin` / `password123`
- **User**: `user` / `userpass`

## Configuration

Create a `.env` file in the project root:

```env
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
LOG_LEVEL=info

# Database Configuration
DATABASE_URL=sqlite:./data/app.db

# Authentication
JWT_SECRET=your-secret-key-here
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.