#!/bin/bash
set -e
# Usage: ./migrate.sh [local|remote] [database_url]

ENV=$1
DB_URL=$2

if [ "$ENV" == "local" ]; then
    echo "Running local SQLite migrations..."
    # Use provided DB_URL or default
    TARGET_DB=${DB_URL:-"sqlite://hyperdashi.db"}
    echo "Target Database: $TARGET_DB"
    
    # Ensure source is correct for SQLite
    sqlx migrate run --source migrations/sqlite --database-url "$TARGET_DB"
    
elif [ "$ENV" == "remote" ]; then
    echo "Running remote Postgres migrations..."
    
    if [ -z "$DB_URL" ]; then
        # Try to read from .env if not provided arg
        if [ -f .env ]; then
            source .env
        fi
        
        # Check if DATABASE_URL is set (from .env or export)
        if [ -z "$DATABASE_URL" ]; then
            echo "Error: DATABASE_URL is not set and not provided as argument."
            echo "Usage: ./migrate.sh remote postgres://user:pass@host/db"
            exit 1
        fi
        TARGET_DB=$DATABASE_URL
    else
        TARGET_DB=$DB_URL
    fi
    
    echo "Target Database: (Hidden for security)"
    
    # Ensure source is correct for Postgres
    sqlx migrate run --source migrations/postgres --database-url "$TARGET_DB"

else
    echo "Usage: ./migrate.sh [local|remote] [database_url]"
    echo ""
    echo "Examples:"
    echo "  ./migrate.sh local                      # Run on local sqlite://hyperdashi.db"
    echo "  ./migrate.sh remote postgres://...      # Run on remote Postgres"
    exit 1
fi

echo "Migration completed successfully!"
