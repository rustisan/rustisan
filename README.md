# Rustisan CLI

Rustisan CLI is a command-line tool inspired by Laravel for building web applications in Rust. This guide provides a comprehensive overview of all available commands and how to use them.

## Table of Contents

1. [Installation](#installation)
2. [Getting Started](#getting-started)
3. [Main Commands](#main-commands)

   * [Create New Project](#create-new-project)
   * [Development Server](#development-server)
   * [Generators](#generators)
   * [Database](#database)
   * [Migrations](#migrations)
   * [Seeders](#seeders)
   * [Cache](#cache)
   * [Queues](#queues)
   * [Configuration](#configuration)
   * [Testing](#testing)
   * [Build & Deploy](#build--deploy)
4. [Project Structure](#project-structure)
5. [Tips & Best Practices](#tips--best-practices)

## Installation

To install Rustisan CLI, run:

```bash
cargo install rustisan-cli
```

## Getting Started

1. Create a new project:

```bash
rustisan new my-project
cd my-project
```

2. Configure your project:

* Edit the `rustisan.toml` file with your settings
* Run `rustisan config:generate-key` to generate your application key

3. Start the development server:

```bash
rustisan serve
```

## Main Commands

### Create New Project

```bash
# Create a basic project
rustisan new project-name

# Create using a specific template
rustisan new project-name --template api

# Create in a specific directory
rustisan new project-name --path /destination/path

# Create without initializing git
rustisan new project-name --git false
```

### Development Server

```bash
# Start default server
rustisan serve

# Set custom host and port
rustisan serve --host 0.0.0.0 --port 8000

# Use specific environment
rustisan serve --env production

# With hot reload
rustisan serve --reload
```

### Generators

The `make` command generates various application components:

```bash
# Controllers
rustisan make:controller UserController
rustisan make:controller UserController --resource  # REST controller
rustisan make:controller UserController --api       # API controller
rustisan make:controller UserController --model User # With model

# Models
rustisan make:model User
rustisan make:model User --migration  # With migration
rustisan make:model User --factory    # With factory
rustisan make:model User --seeder     # With seeder

# Migrations
rustisan make:migration create_users_table
rustisan make:migration add_field_to_users --table users

# Other components
rustisan make:middleware AuthMiddleware
rustisan make:request CreateUserRequest
rustisan make:resource UserResource
rustisan make:seeder UsersSeeder
rustisan make:factory UserFactory
rustisan make:job ProcessEmailJob
rustisan make:event UserCreated
rustisan make:listener SendWelcomeEmail
```

### Database

```bash
# Show database status
rustisan db:status

# Create database
rustisan db:create

# Drop database
rustisan db:drop --force

# Reset (drop + create)
rustisan db:reset --force

# Seed database
rustisan db:seed
```

### Migrations

```bash
# Run pending migrations
rustisan migrate

# Rollback last migration
rustisan migrate down

# Rollback multiple migrations
rustisan migrate down --steps 3

# Full reset
rustisan migrate:reset

# Show migration status
rustisan migrate:status
```

### Seeders

```bash
# Run all seeders
rustisan seed

# Run specific seeder
rustisan seed --class UsersSeeder

# Force in production
rustisan seed --force
```

### Cache

```bash
# Clear all cache
rustisan cache:clear

# Clear specific key
rustisan cache:forget cache-key

# Cache configuration
rustisan cache:config
```

### Queues

```bash
# Start queue worker
rustisan queue:work

# Configure worker
rustisan queue:work --queue emails --max-jobs 1000

# View failed jobs
rustisan queue:failed

# Retry failed jobs
rustisan queue:retry
rustisan queue:retry job-id
```

### Configuration

```bash
# Show configuration
rustisan config:show

# Get specific value
rustisan config:get app.name

# Set value
rustisan config:set app.name "New App"

# Generate application key
rustisan config:generate-key
```

### Testing

```bash
# Run all tests
rustisan test

# Run specific tests
rustisan test users
rustisan test --unit
rustisan test --integration
```

### Build & Deploy

```bash
# Production build
rustisan build --env production

# Optimized build
rustisan build --optimize

# Deploy
rustisan deploy
rustisan deploy staging --skip-build
rustisan deploy --dry-run
```

## Project Structure

A Rustisan project follows this directory structure:

```
.
├── src/
│   ├── controllers/     # HTTP controllers
│   ├── models/          # Data models
│   ├── middleware/      # HTTP middleware
│   ├── requests/        # Request validators
│   ├── resources/       # API resources
│   ├── services/        # Services
│   ├── jobs/            # Background jobs
│   ├── events/          # Events
│   └── listeners/       # Event listeners
├── database/
│   ├── migrations/      # Migrations
│   ├── seeders/         # Seeders
│   └── factories/       # Factories
├── routes/              # Route definitions
├── resources/           # Views and assets
├── storage/             # Storage
├── tests/               # Tests
├── rustisan.toml        # Configuration
└── Cargo.toml           # Dependencies
```

## Tips & Best Practices

1. **Configuration**

   * Keep passwords and sensitive keys in environment variables
   * Use different settings for dev/prod in `rustisan.toml`

2. **Development**

   * Use `rustisan serve --reload` during development
   * Take advantage of generators for code consistency
   * Write tests for critical components

3. **Database**

   * Always use migrations for DB changes
   * Keep seeders updated for test data
   * Backup before using `migrate:reset`

4. **Deploy**

   * Use `rustisan build --optimize` for production
   * Validate configs with `rustisan config:validate`
   * Test deploys using `--dry-run`

5. **Performance**

   * Use cache for frequently accessed data
   * Properly configure queue workers
   * Monitor logs and metrics in production

## Additional Resources

* [Official Documentation](https://github.com/rustisan/rustisan)
* [Project Examples](https://github.com/rustisan/rustisan-examples)
* [Recommended Packages](https://github.com/rustisan/rustisan-packages)

## Support

For support and discussions:

* GitHub Issues: [rustisan/issues](https://github.com/rustisan/rustisan/issues)
* Discord: [Rustisan Channel](https://discord.gg/cznSqvX2EM)

---

If you'd like this as a Markdown file or want help publishing it to your GitHub repo or documentation site, let me know!
