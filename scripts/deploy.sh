#!/bin/bash

# Simple Deployment Script for Immutable Encryption System
# This script sets up the basic system for development/testing

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️ $1${NC}"
}

# Check if Docker is installed
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    print_status "Docker is installed"
    
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed. Please install Docker Compose."
        exit 1
    fi
    print_status "Docker Compose is installed"
}

# Create necessary directories
create_directories() {
    print_info "Creating necessary directories..."
    
    mkdir -p data/uploads
    mkdir -p data/backups
    mkdir -p data/logs
    mkdir -p keys/tls
    
    # Set proper permissions
    chmod 755 data
    chmod 755 keys
    chmod 700 keys/tls
    
    print_status "Directories created"
}

# Generate basic configuration
create_config() {
    print_info "Creating basic configuration..."
    
    # Create environment file
    if [ ! -f .env ]; then
        cat > .env << EOF
# Immutable Encryption Environment Configuration
API_BASE_URL=http://localhost:8000
AUTH_TOKEN=demo-token
RUST_LOG=info
REDIS_URL=redis://redis:6379
DATABASE_URL=postgresql://postgres:password@postgres:5432/immutable_encryption
EOF
        print_status "Environment file created"
    else
        print_warning "Environment file already exists"
    fi
    
    # Generate TLS certificates for testing
    if [ ! -f keys/tls/server.crt ]; then
        print_info "Generating TLS certificates..."
        openssl req -x509 -newkey rsa:2048 -keyout keys/tls/server.key -out keys/tls/server.crt -days 365 -nodes -subj "/C=US/ST=State/L=City/O=Test/CN=localhost" 2>/dev/null || {
            print_warning "OpenSSL not available, skipping certificate generation"
        }
        print_status "TLS certificates generated"
    fi
    
    # Generate basic config file
    if [ ! -f config.toml ]; then
        cat > config.toml << EOF
[server]
host = "0.0.0.0"
port = 8080
max_connections = 1000
request_timeout_ms = 30000

[encryption]
primary_key_path = "keys/primary.key"
key_rotation_interval_seconds = 3600
quantum_resistant = true
hardware_backed = false
compression_enabled = true

[blockchain.ethereum]
rpc_url = "https://mainnet.infura.io/v3/YOUR_PROJECT_ID"
gas_limit = 100000
gas_price_gwei = 20.0
confirmations_required = 12

[blockchain.bitcoin]
rpc_url = "https://blockstream.info/api"
wallet_name = "evidence_wallet"
fee_sat_per_byte = 10
confirmations_required = 6

[storage]
database_path = "data/blockchain.db"
ipfs_enabled = true
ipfs_api_url = "http://localhost:5001"
backup_enabled = true
backup_path = "data/backups"
retention_days = 2555

[verification]
strict_mode = true
quantum_verification = true
hardware_attestation = false

[logging]
level = "info"
max_file_size_mb = 100
max_files = 10
EOF
        print_status "Configuration file created"
    fi
}

# Build and start services
deploy_services() {
    print_info "Building and starting services..."
    
    # Pull latest images
    docker-compose -f docker/docker-compose.yml pull
    
    # Build custom images
    docker-compose -f docker/docker-compose.yml build
    
    # Start services
    docker-compose -f docker/docker-compose.yml up -d
    
    print_status "Services deployed"
}

# Wait for services to be ready
wait_for_services() {
    print_info "Waiting for services to be ready..."
    
    # Wait for Python API
    for i in {1..30}; do
        if curl -f http://localhost:8000/health &>/dev/null; then
            print_status "Python API is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            print_error "Python API failed to start within 60 seconds"
            return 1
        fi
        sleep 2
    done
    
    # Wait for Streamlit
    for i in {1..30}; do
        if curl -f http://localhost:8501 &>/dev/null; then
            print_status "Streamlit is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            print_warning "Streamlit may still be starting"
        fi
        sleep 2
    done
    
    # Check other services
    if docker-compose -f docker/docker-compose.yml ps | grep -q "Up"; then
        print_status "Services are running"
    else
        print_error "Some services failed to start"
        docker-compose -f docker/docker-compose.yml ps
        return 1
    fi
}

# Show deployment information
show_info() {
    print_info "Deployment completed successfully!"
    echo ""
    echo "Service URLs:"
    echo "  🦀 Rust Backend:    http://localhost:8080"
    echo "  🐍 Python API:       http://localhost:8000"
    echo "  🌐 Streamlit UI:     http://localhost:8501"
    echo "  📊 API Docs:         http://localhost:8000/docs"
    echo "  ❤️  Health Check:      http://localhost:8000/health"
    echo ""
    echo "Additional Services:"
    echo "  🗄️  PostgreSQL:       localhost:5432"
    echo "  🔴 Redis:           localhost:6379"
    echo "  🌐 IPFS:           localhost:5001"
    echo "  📈 Grafana:         http://localhost:3000 (if monitoring enabled)"
    echo ""
    echo "Commands:"
    echo "  📋 View logs:        docker-compose -f docker/docker-compose.yml logs -f"
    echo "  🛑 Stop services:     docker-compose -f docker/docker-compose.yml down"
    echo "  🔄 Restart services:  docker-compose -f docker/docker-compose.yml restart"
    echo ""
    echo "Default Authentication:"
    echo "  🗝️  Token:            demo-token"
    echo ""
}

# Cleanup function
cleanup() {
    print_info "Cleaning up..."
    docker-compose -f docker/docker-compose.yml down -v
    docker system prune -f
    print_status "Cleanup completed"
}

# Show logs
show_logs() {
    print_info "Showing service logs..."
    docker-compose -f docker/docker-compose.yml logs -f
}

# Main deployment function
main() {
    case "${1:-deploy}" in
        "deploy"|"")
            check_docker
            create_directories
            create_config
            deploy_services
            wait_for_services
            show_info
            ;;
        "cleanup"|"clean")
            cleanup
            ;;
        "logs"|"log")
            show_logs
            ;;
        "restart")
            print_info "Restarting services..."
            docker-compose -f docker/docker-compose.yml restart
            wait_for_services
            show_info
            ;;
        "status"|"ps")
            print_info "Service status:"
            docker-compose -f docker/docker-compose.yml ps
            ;;
        "help"|"-h"|"--help")
            echo "Usage: $0 [command]"
            echo ""
            echo "Commands:"
            echo "  deploy     Deploy the full system (default)"
            echo "  cleanup    Stop and remove all containers and data"
            echo "  logs       Show service logs"
            echo "  restart    Restart all services"
            echo "  status     Show service status"
            echo "  help       Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0              # Deploy system"
            echo "  $0 deploy        # Deploy system"
            echo "  $0 logs          # Show logs"
            echo "  $0 cleanup       # Stop and clean"
            ;;
        *)
            print_error "Unknown command: $1"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"