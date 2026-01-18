#!/bin/bash

# Real-Time Immutable Encryption System - Deployment Script
# This script sets up the complete system for production deployment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="immutable-encryption"
DOCKER_REGISTRY="your-registry.com"
VERSION=${1:-"latest"}

echo -e "${BLUE}🚀 Deploying ${PROJECT_NAME} version ${VERSION}${NC}"

# Function to print colored status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check prerequisites
check_prerequisites() {
    echo -e "${BLUE}🔍 Checking prerequisites...${NC}"
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed"
        exit 1
    fi
    print_status "Docker is installed"
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed"
        exit 1
    fi
    print_status "Docker Compose is installed"
    
    # Check kubectl if deploying to Kubernetes
    if command -v kubectl &> /dev/null; then
        print_status "kubectl is available"
    else
        print_warning "kubectl not found - Kubernetes deployment will not be available"
    fi
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed"
        exit 1
    fi
    print_status "Rust/Cargo is installed"
    
    # Check Python
    if ! command -v python3 &> /dev/null; then
        print_error "Python 3 is not installed"
        exit 1
    fi
    print_status "Python 3 is installed"
    
    # Check Node.js (for Streamlit)
    if ! command -v node &> /dev/null; then
        print_warning "Node.js not found - some optional features may not work"
    else
        print_status "Node.js is installed"
    fi
}

# Build Rust components
build_rust() {
    echo -e "${BLUE}🦀 Building Rust components...${NC}"
    
    cd "$(dirname "$0")/.."
    
    # Build release version
    cargo build --release
    
    if [ $? -eq 0 ]; then
        print_status "Rust build completed successfully"
    else
        print_error "Rust build failed"
        exit 1
    fi
}

# Build Python components
build_python() {
    echo -e "${BLUE}🐍 Building Python components...${NC}"
    
    cd "$(dirname "$0")/.."
    
    # Create virtual environment if it doesn't exist
    if [ ! -d "python_api/venv" ]; then
        python3 -m venv python_api/venv
        print_status "Created Python virtual environment"
    fi
    
    # Activate virtual environment and install dependencies
    source python_api/venv/bin/activate
    pip install -r requirements.txt
    
    if [ $? -eq 0 ]; then
        print_status "Python dependencies installed successfully"
    else
        print_error "Python dependency installation failed"
        exit 1
    fi
    
    # Install additional development dependencies
    pip install pytest pytest-asyncio black mypy ruff
    
    print_status "Python setup completed"
}

# Run tests
run_tests() {
    echo -e "${BLUE}🧪 Running tests...${NC}"
    
    cd "$(dirname "$0")/.."
    
    # Rust tests
    echo "Running Rust tests..."
    cargo test --release
    
    if [ $? -ne 0 ]; then
        print_error "Rust tests failed"
        exit 1
    fi
    print_status "Rust tests passed"
    
    # Python tests
    echo "Running Python tests..."
    source python_api/venv/bin/activate
    cd python_api
    python -m pytest tests/ -v
    
    if [ $? -ne 0 ]; then
        print_error "Python tests failed"
        exit 1
    fi
    print_status "Python tests passed"
    
    cd ..
}

# Build Docker images
build_docker_images() {
    echo -e "${BLUE}🐳 Building Docker images...${NC}"
    
    cd "$(dirname "$0")/.."
    
    # Build Rust backend image
    docker build -t ${DOCKER_REGISTRY}/${PROJECT_NAME}-rust-backend:${VERSION} -f docker/rust-backend.Dockerfile .
    
    if [ $? -eq 0 ]; then
        print_status "Rust backend image built successfully"
    else
        print_error "Rust backend image build failed"
        exit 1
    fi
    
    # Build Python API image
    docker build -t ${DOCKER_REGISTRY}/${PROJECT_NAME}-python-api:${VERSION} -f docker/python-api.Dockerfile .
    
    if [ $? -eq 0 ]; then
        print_status "Python API image built successfully"
    else
        print_error "Python API image build failed"
        exit 1
    fi
    
    # Build Streamlit frontend image
    docker build -t ${DOCKER_REGISTRY}/${PROJECT_NAME}-streamlit:${VERSION} -f docker/streamlit.Dockerfile python_api/streamlit/
    
    if [ $? -eq 0 ]; then
        print_status "Streamlit frontend image built successfully"
    else
        print_error "Streamlit frontend image build failed"
        exit 1
    fi
}

# Deploy with Docker Compose
deploy_docker_compose() {
    echo -e "${BLUE}🐙 Deploying with Docker Compose...${NC}"
    
    cd "$(dirname "$0")/.."
    
    # Create necessary directories
    mkdir -p data/uploads
    mkdir -p data/backups
    mkdir -p data/logs
    mkdir -p keys
    
    # Set up environment file
    if [ ! -f ".env" ]; then
        cp .env.example .env
        print_warning "Created .env file from template - please review and update"
    fi
    
    # Deploy
    docker-compose -f docker/docker-compose.yml up -d
    
    if [ $? -eq 0 ]; then
        print_status "Docker Compose deployment successful"
        
        # Wait for services to be ready
        echo -e "${BLUE}⏳ Waiting for services to be ready...${NC}"
        sleep 30
        
        # Check service health
        if curl -f http://localhost:8000/health &>/dev/null; then
            print_status "Python API is healthy"
        else
            print_warning "Python API may not be ready yet"
        fi
        
        if curl -f http://localhost:8501 &>/dev/null; then
            print_status "Streamlit frontend is accessible"
        else
            print_warning "Streamlit frontend may not be ready yet"
        fi
        
    else
        print_error "Docker Compose deployment failed"
        exit 1
    fi
}

# Deploy to Kubernetes
deploy_kubernetes() {
    echo -e "${BLUE}☸️ Deploying to Kubernetes...${NC}"
    
    if ! command -v kubectl &> /dev/null; then
        print_error "kubectl is not available for Kubernetes deployment"
        return
    fi
    
    cd "$(dirname "$0")/.."
    
    # Create namespace
    kubectl create namespace ${PROJECT_NAME} --dry-run=client -o yaml | kubectl apply -f -
    
    # Apply ConfigMaps
    kubectl apply -f k8s/configmaps/ --namespace=${PROJECT_NAME}
    
    # Apply Secrets
    kubectl apply -f k8s/secrets/ --namespace=${PROJECT_NAME}
    
    # Apply Persistent Volumes
    kubectl apply -f k8s/pv/ --namespace=${PROJECT_NAME}
    
    # Apply Deployments
    kubectl apply -f k8s/deployments/ --namespace=${PROJECT_NAME}
    
    # Apply Services
    kubectl apply -f k8s/services/ --namespace=${PROJECT_NAME}
    
    # Apply Ingress
    kubectl apply -f k8s/ingress/ --namespace=${PROJECT_NAME}
    
    print_status "Kubernetes deployment completed"
    
    # Wait for deployments to be ready
    echo -e "${BLUE}⏳ Waiting for Kubernetes deployments to be ready...${NC}"
    kubectl wait --for=condition=available --timeout=300s deployment --all --namespace=${PROJECT_NAME}
    
    # Show status
    kubectl get pods --namespace=${PROJECT_NAME}
    kubectl get services --namespace=${PROJECT_NAME}
}

# Setup monitoring
setup_monitoring() {
    echo -e "${BLUE}📊 Setting up monitoring...${NC}"
    
    cd "$(dirname "$0")/.."
    
    # Deploy Prometheus and Grafana if using Docker Compose
    if [ -f "docker/docker-compose.monitoring.yml" ]; then
        docker-compose -f docker/docker-compose.monitoring.yml up -d
        print_status "Monitoring stack deployed"
    fi
    
    # Deploy monitoring to Kubernetes
    if command -v kubectl &> /dev/null && [ -d "k8s/monitoring" ]; then
        kubectl apply -f k8s/monitoring/ --namespace=${PROJECT_NAME}
        print_status "Kubernetes monitoring deployed"
    fi
}

# Security hardening
security_hardening() {
    echo -e "${BLUE}🔒 Applying security hardening...${NC}"
    
    cd "$(dirname "$0")/.."
    
    # Generate secure keys if they don't exist
    if [ ! -f "keys/primary.key" ]; then
        openssl rand -hex 32 > keys/primary.key
        chmod 600 keys/primary.key
        print_status "Generated primary encryption key"
    fi
    
    # Set up TLS certificates
    if [ ! -d "keys/tls" ]; then
        mkdir -p keys/tls
        openssl req -x509 -newkey rsa:4096 -keyout keys/tls/server.key -out keys/tls/server.crt -days 365 -nodes -subj "/C=US/ST=State/L=City/O=Organization/CN=localhost"
        chmod 600 keys/tls/server.key
        print_status "Generated TLS certificates"
    fi
    
    # Set proper file permissions
    chmod 755 scripts/deploy.sh
    chmod -R 755 docker/
    chmod -R 755 k8s/
    
    print_status "Security hardening completed"
}

# Cleanup old deployments
cleanup() {
    echo -e "${BLUE}🧹 Cleaning up old deployments...${NC}"
    
    # Stop and remove Docker containers
    docker-compose -f docker/docker-compose.yml down -v 2>/dev/null || true
    docker-compose -f docker/docker-compose.monitoring.yml down -v 2>/dev/null || true
    
    # Remove old Docker images
    docker rmi ${DOCKER_REGISTRY}/${PROJECT_NAME}-rust-backend:${VERSION} 2>/dev/null || true
    docker rmi ${DOCKER_REGISTRY}/${PROJECT_NAME}-python-api:${VERSION} 2>/dev/null || true
    docker rmi ${DOCKER_REGISTRY}/${PROJECT_NAME}-streamlit:${VERSION} 2>/dev/null || true
    
    print_status "Cleanup completed"
}

# Show deployment info
show_deployment_info() {
    echo -e "${BLUE}📋 Deployment Information:${NC}"
    echo -e "${GREEN}Rust Backend:${NC} http://localhost:8080"
    echo -e "${GREEN}Python API:${NC} http://localhost:8000"
    echo -e "${GREEN}Streamlit Frontend:${NC} http://localhost:8501"
    echo -e "${GREEN}Health Check:${NC} http://localhost:8000/health"
    echo -e "${GREEN}API Documentation:${NC} http://localhost:8000/docs"
    echo ""
    echo -e "${BLUE}Monitoring (if enabled):${NC}"
    echo -e "${GREEN}Prometheus:${NC} http://localhost:9090"
    echo -e "${GREEN}Grafana:${NC} http://localhost:3000 (admin/admin)"
    echo ""
    echo -e "${BLUE}Logs:${NC}"
    echo "Rust Backend: docker-compose logs rust-backend"
    echo "Python API: docker-compose logs python-api"
    echo "Streamlit: docker-compose logs streamlit"
}

# Main deployment function
main() {
    echo -e "${BLUE}🚀 Starting deployment of ${PROJECT_NAME} v${VERSION}${NC}"
    
    # Parse command line arguments
    DEPLOYMENT_TYPE=${2:-"docker-compose"}
    SKIP_TESTS=${3:-"false"}
    
    case $DEPLOYMENT_TYPE in
        "docker-compose"|"local")
            check_prerequisites
            cleanup
            
            if [ "$SKIP_TESTS" != "true" ]; then
                build_rust
                build_python
                run_tests
            fi
            
            security_hardening
            build_docker_images
            deploy_docker_compose
            setup_monitoring
            show_deployment_info
            ;;
            
        "kubernetes"|"k8s")
            check_prerequisites
            cleanup
            
            if [ "$SKIP_TESTS" != "true" ]; then
                build_rust
                build_python
                run_tests
            fi
            
            security_hardening
            build_docker_images
            deploy_kubernetes
            setup_monitoring
            show_deployment_info
            ;;
            
        "build-only")
            check_prerequisites
            build_rust
            build_python
            run_tests
            build_docker_images
            ;;
            
        "cleanup")
            cleanup
            ;;
            
        "help"|"-h"|"--help")
            echo "Usage: $0 [VERSION] [DEPLOYMENT_TYPE] [SKIP_TESTS]"
            echo ""
            echo "VERSION: Docker image version (default: latest)"
            echo "DEPLOYMENT_TYPE: docker-compose, kubernetes, build-only, cleanup"
            echo "SKIP_TESTS: true or false (default: false)"
            echo ""
            echo "Examples:"
            echo "  $0 v1.0.0 docker-compose"
            echo "  $0 v1.0.0 kubernetes"
            echo "  $0 v1.0.0 build-only true"
            ;;
            
        *)
            print_error "Unknown deployment type: $DEPLOYMENT_TYPE"
            echo "Use 'help' for usage information"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"