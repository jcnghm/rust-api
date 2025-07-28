# API Framework - Built in Rust Actix Web

A robust, secure REST API framework built with Rust and Actix Web, featuring JWT authentication, OAuth integration, and comprehensive CRUD operations.

## Features

**Authentication & Authorization**
- JWT-based authentication with custom middleware
- OAuth integration for third-party authentication
- Role-based access control (admin/user roles)
- Secure token validation and refresh

**API Capabilities**
- RESTful CRUD operations for objects
- Health check endpoints
- Request/response logging
- Structured error handling
- Environment-based configuration

**Architecture**
- Clean separation of concerns
- Modular handler organization
- Reusable middleware components
- Repository pattern for data access

## Quick Start

### Prerequisites
- Rust 1.70+ and Cargo
- Environment variables configured (see `.env.example`)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd rust-api-framework

# Copy environment template
cp .env.example .env

# Install dependencies and run
cargo run
```

The server will start at `http://127.0.0.1:8080` by default.

## API Endpoints

### Authentication
- `POST /token` - Login with username/password
- `GET /oauth/login` - Initiate OAuth flow
- `GET /oauth/callback` - OAuth callback handler

### Public Endpoints
- `GET /health` - Health check

### Protected Endpoints
All endpoints below require `Authorization: Bearer <token>` header:

- `GET /objects` - List all objects
- `GET /objects/{id}` - Get specific object
- `POST /objects` - Create new object
- `PUT /objects/{id}` - Update object
- `PATCH /objects/{id}` - Partial update
- `DELETE /objects/{id}` - Delete object
- `GET /objects/{id}/profile` - Get object profile
- `GET /stats` - Get system statistics

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

### OAuth Flow
Visit `/oauth/login` to initiate OAuth authentication with your configured provider.

## Configuration

Create a `.env` file in the project root:

```env
# Server Configuration
SERVER_ADDRESS=127.0.0.1:8080
LOG_LEVEL=info

# Authentication
JWT_SECRET=your-super-secret-jwt-key
JWT_EXPIRATION=3600

# OAuth (optional)
OAUTH_CLIENT_ID=your-oauth-client-id
OAUTH_CLIENT_SECRET=your-oauth-client-secret
OAUTH_REDIRECT_URI=http://localhost:8080/oauth/callback
```

## Development

### Running Tests
```bash
cargo test
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

### Running with Different Log Levels
```bash
RUST_LOG=debug cargo run
```

## API Testing

### Using curl
```bash
# Get authentication token
TOKEN=$(curl -s -X POST http://localhost:8080/token \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password123"}' \
  | jq -r '.token')

# Create an object
curl -X POST http://localhost:8080/objects \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Object", "description": "A test object"}'
```

### Using Postman
Import the provided Postman collection and environment variables for comprehensive API testing.

## Deployment

### Docker (Optional)
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust-api-framework /usr/local/bin/
EXPOSE 8080
CMD ["rust-api-framework"]
```

### Environment Setup
Ensure all required environment variables are set in your deployment environment.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with:
- [Actix Web](https://actix.rs/) - Fast, powerful web framework
- [serde](https://serde.rs/) - Serialization framework
- [jsonwebtoken](https://github.com/Keats/jsonwebtoken) - JWT implementation
- [env_logger](https://github.com/rust-cli/env_logger) - Logging implementation
