#!/bin/bash

# Constellation Backup and Recovery Script
# This script provides automated backup and recovery operations for Constellation

set -e

# Configuration
BACKUP_DIR="${BACKUP_DIR:-/var/backups/constellation}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"
DATABASE_HOST="${DATABASE_HOST:-localhost}"
DATABASE_PORT="${DATABASE_PORT:-5432}"
DATABASE_NAME="${DATABASE_NAME:-constellation}"
DATABASE_USER="${DATABASE_USER:-constellation}"
REDIS_HOST="${REDIS_HOST:-localhost}"
REDIS_PORT="${REDIS_PORT:-6379}"
IGGY_DATA_DIR="${IGGY_DATA_DIR:-/var/lib/iggy}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check for required commands
    for cmd in pg_dump redis-cli tar gzip; do
        if ! command -v $cmd &> /dev/null; then
            log_error "$cmd is required but not installed"
            exit 1
        fi
    done
    
    # Create backup directory if it doesn't exist
    mkdir -p "$BACKUP_DIR"
    
    log_info "Prerequisites check passed"
}

# Backup PostgreSQL database
backup_database() {
    local timestamp=$(date '+%Y%m%d_%H%M%S')
    local backup_file="$BACKUP_DIR/database_${timestamp}.sql.gz"
    
    log_info "Starting PostgreSQL database backup..."
    
    # Set PGPASSWORD if provided
    if [ -n "$DATABASE_PASSWORD" ]; then
        export PGPASSWORD="$DATABASE_PASSWORD"
    fi
    
    # Perform backup
    if pg_dump -h "$DATABASE_HOST" -p "$DATABASE_PORT" -U "$DATABASE_USER" \
        -d "$DATABASE_NAME" --clean --if-exists | gzip > "$backup_file"; then
        log_info "Database backup completed: $backup_file"
        echo "$backup_file"
    else
        log_error "Database backup failed"
        return 1
    fi
}

# Backup Redis data
backup_redis() {
    local timestamp=$(date '+%Y%m%d_%H%M%S')
    local backup_file="$BACKUP_DIR/redis_${timestamp}.rdb"
    
    log_info "Starting Redis backup..."
    
    # Save Redis data
    if redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" SAVE &> /dev/null; then
        # Copy the RDB file
        local rdb_file=$(redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" CONFIG GET dir | tail -n 1)/dump.rdb
        if [ -f "$rdb_file" ]; then
            cp "$rdb_file" "$backup_file"
            log_info "Redis backup completed: $backup_file"
            echo "$backup_file"
        else
            log_error "Redis RDB file not found"
            return 1
        fi
    else
        log_error "Redis backup failed"
        return 1
    fi
}

# Backup Iggy message broker data
backup_iggy() {
    local timestamp=$(date '+%Y%m%d_%H%M%S')
    local backup_file="$BACKUP_DIR/iggy_${timestamp}.tar.gz"
    
    log_info "Starting Iggy message broker backup..."
    
    if [ -d "$IGGY_DATA_DIR" ]; then
        # Create tar archive of Iggy data directory
        if tar -czf "$backup_file" -C "$(dirname "$IGGY_DATA_DIR")" "$(basename "$IGGY_DATA_DIR")" 2>/dev/null; then
            log_info "Iggy backup completed: $backup_file"
            echo "$backup_file"
        else
            log_error "Iggy backup failed"
            return 1
        fi
    else
        log_warn "Iggy data directory not found: $IGGY_DATA_DIR"
        return 0
    fi
}

# Backup configuration files
backup_config() {
    local timestamp=$(date '+%Y%m%d_%H%M%S')
    local backup_file="$BACKUP_DIR/config_${timestamp}.tar.gz"
    
    log_info "Starting configuration backup..."
    
    # List of configuration files to backup
    local config_files=(
        "/etc/constellation"
        "/var/lib/constellation/config"
        "config.yaml"
        "secrets.yaml"
    )
    
    # Filter existing files
    local existing_files=()
    for file in "${config_files[@]}"; do
        if [ -e "$file" ]; then
            existing_files+=("$file")
        fi
    done
    
    if [ ${#existing_files[@]} -gt 0 ]; then
        if tar -czf "$backup_file" "${existing_files[@]}" 2>/dev/null; then
            log_info "Configuration backup completed: $backup_file"
            echo "$backup_file"
        else
            log_error "Configuration backup failed"
            return 1
        fi
    else
        log_warn "No configuration files found to backup"
        return 0
    fi
}

# Create full backup
create_full_backup() {
    log_info "Starting full backup of Constellation..."
    
    local timestamp=$(date '+%Y%m%d_%H%M%S')
    local backup_dir="$BACKUP_DIR/full_${timestamp}"
    
    mkdir -p "$backup_dir"
    
    # Backup individual components
    local database_backup=$(backup_database) || return 1
    local redis_backup=$(backup_redis) || return 1
    local iggy_backup=$(backup_iggy) || return 1
    local config_backup=$(backup_config) || return 1
    
    # Create manifest
    cat > "$backup_dir/manifest.json" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "version": "1.0.0",
  "components": {
    "database": "$(basename "$database_backup")",
    "redis": "$(basename "$redis_backup")",
    "iggy": "$(basename "$iggy_backup")",
    "config": "$(basename "$config_backup")"
  },
  "system_info": {
    "hostname": "$(hostname)",
    "backup_dir": "$BACKUP_DIR"
  }
}
EOF
    
    # Move backups to the full backup directory
    [ -n "$database_backup" ] && mv "$database_backup" "$backup_dir/"
    [ -n "$redis_backup" ] && mv "$redis_backup" "$backup_dir/"
    [ -n "$iggy_backup" ] && mv "$iggy_backup" "$backup_dir/"
    [ -n "$config_backup" ] && mv "$config_backup" "$backup_dir/"
    
    # Create archive
    local archive_file="$BACKUP_DIR/full_backup_${timestamp}.tar.gz"
    tar -czf "$archive_file" -C "$BACKUP_DIR" "full_${timestamp}"
    rm -rf "$backup_dir"
    
    log_info "Full backup completed: $archive_file"
    echo "$archive_file"
}

# Restore from backup
restore_backup() {
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
        return 1
    fi
    
    log_info "Starting restore from backup: $backup_file"
    
    # Extract backup
    local temp_dir=$(mktemp -d)
    tar -xzf "$backup_file" -C "$temp_dir"
    
    # Read manifest
    local manifest="$temp_dir/full_*/manifest.json"
    if [ ! -f "$manifest" ]; then
        log_error "Manifest not found in backup"
        rm -rf "$temp_dir"
        return 1
    fi
    
    # Parse manifest
    local database_file=$(jq -r '.components.database' "$manifest")
    local redis_file=$(jq -r '.components.redis' "$manifest")
    local iggy_file=$(jq -r '.components.iggy' "$manifest")
    local config_file=$(jq -r '.components.config' "$manifest")
    
    log_info "Restoring components from backup..."
    
    # Restore database
    if [ "$database_file" != "null" ] && [ -f "$temp_dir/full_*/$database_file" ]; then
        log_info "Restoring PostgreSQL database..."
        gunzip -c "$temp_dir/full_*/$database_file" | psql -h "$DATABASE_HOST" -p "$DATABASE_PORT" -U "$DATABASE_USER" -d "$DATABASE_NAME"
    fi
    
    # Restore Redis
    if [ "$redis_file" != "null" ] && [ -f "$temp_dir/full_*/$redis_file" ]; then
        log_info "Restoring Redis data..."
        # Stop Redis, replace RDB file, start Redis
        systemctl stop redis || true
        cp "$temp_dir/full_*/$redis_file" "$(redis-cli CONFIG GET dir | tail -n 1)/dump.rdb"
        systemctl start redis || true
    fi
    
    # Restore Iggy
    if [ "$iggy_file" != "null" ] && [ -f "$temp_dir/full_*/$iggy_file" ]; then
        log_info "Restoring Iggy message broker data..."
        # Stop Iggy, restore data, start Iggy
        systemctl stop iggy || true
        tar -xzf "$temp_dir/full_*/$iggy_file" -C /
        systemctl start iggy || true
    fi
    
    # Restore configuration
    if [ "$config_file" != "null" ] && [ -f "$temp_dir/full_*/$config_file" ]; then
        log_info "Restoring configuration files..."
        tar -xzf "$temp_dir/full_*/$config_file" -C /
    fi
    
    # Cleanup
    rm -rf "$temp_dir"
    
    log_info "Restore completed successfully"
}

# Clean up old backups
cleanup_old_backups() {
    log_info "Cleaning up backups older than $RETENTION_DAYS days..."
    
    find "$BACKUP_DIR" -name "*.tar.gz" -type f -mtime +$RETENTION_DAYS -delete
    find "$BACKUP_DIR" -name "*.sql.gz" -type f -mtime +$RETENTION_DAYS -delete
    find "$BACKUP_DIR" -name "*.rdb" -type f -mtime +$RETENTION_DAYS -delete
    
    log_info "Cleanup completed"
}

# List available backups
list_backups() {
    log_info "Available backups in $BACKUP_DIR:"
    
    echo "Full backups:"
    find "$BACKUP_DIR" -name "full_backup_*.tar.gz" -type f -printf "%f\n" | sort
    
    echo ""
    echo "Database backups:"
    find "$BACKUP_DIR" -name "database_*.sql.gz" -type f -printf "%f\n" | sort
    
    echo ""
    echo "Redis backups:"
    find "$BACKUP_DIR" -name "redis_*.rdb" -type f -printf "%f\n" | sort
}

# Verify backup integrity
verify_backup() {
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
        return 1
    fi
    
    log_info "Verifying backup integrity: $backup_file"
    
    # Check if tar archive is valid
    if tar -tzf "$backup_file" &> /dev/null; then
        log_info "Backup archive is valid"
        
        # Extract and check manifest
        local temp_dir=$(mktemp -d)
        tar -xzf "$backup_file" -C "$temp_dir" --wildcards "*/manifest.json" 2>/dev/null
        
        if [ -f "$temp_dir"/*/manifest.json ]; then
            log_info "Manifest found and valid"
            rm -rf "$temp_dir"
            return 0
        else
            log_error "Manifest not found in backup"
            rm -rf "$temp_dir"
            return 1
        fi
    else
        log_error "Backup archive is corrupt"
        return 1
    fi
}

# Show usage
show_usage() {
    cat << EOF
Constellation Backup and Recovery Script

Usage: $0 [command]

Commands:
  backup           Create a full backup of all components
  restore <file>   Restore from a backup file
  list             List available backups
  verify <file>    Verify backup integrity
  cleanup          Clean up old backups
  help             Show this help message

Environment Variables:
  BACKUP_DIR       Backup directory (default: /var/backups/constellation)
  RETENTION_DAYS   Number of days to keep backups (default: 30)
  DATABASE_HOST    PostgreSQL host (default: localhost)
  DATABASE_PORT    PostgreSQL port (default: 5432)
  DATABASE_NAME    Database name (default: constellation)
  DATABASE_USER    Database user (default: constellation)
  DATABASE_PASSWORD Database password (required for backup/restore)
  REDIS_HOST       Redis host (default: localhost)
  REDIS_PORT       Redis port (default: 6379)
  IGGY_DATA_DIR    Iggy data directory (default: /var/lib/iggy)

Examples:
  # Create a full backup
  DATABASE_PASSWORD=secret $0 backup
  
  # List available backups
  $0 list
  
  # Restore from backup
  DATABASE_PASSWORD=secret $0 restore /var/backups/constellation/full_backup_20250115_103000.tar.gz
  
  # Clean up old backups
  $0 cleanup
EOF
}

# Main execution
main() {
    check_prerequisites
    
    case "${1:-help}" in
        backup)
            create_full_backup
            cleanup_old_backups
            ;;
        restore)
            if [ -z "$2" ]; then
                log_error "Restore requires a backup file"
                show_usage
                exit 1
            fi
            restore_backup "$2"
            ;;
        list)
            list_backups
            ;;
        verify)
            if [ -z "$2" ]; then
                log_error "Verify requires a backup file"
                show_usage
                exit 1
            fi
            verify_backup "$2"
            ;;
        cleanup)
            cleanup_old_backups
            ;;
        help|--help|-h)
            show_usage
            ;;
        *)
            log_error "Unknown command: $1"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"