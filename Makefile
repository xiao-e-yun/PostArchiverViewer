# Paths
FRONTEND_DIR := frontend

# Commands
CARGO := cargo
BUN := bun

# Targets
.PHONY: all build clean dev backend frontend

## Build both backend and frontend
all: build

## Build everything
build: frontend backend 

## Build Rust backend (in root)
backend:
	$(CARGO) build --release

## Build frontend with Bun
frontend:
	cd $(FRONTEND_DIR) && $(BUN) run build

## Run frontend dev server
dev-frontend:
	cd $(FRONTEND_DIR) && $(BUN) run dev

## Run backend in dev mode
dev-backend:
	$(CARGO) run

## Clean both builds
clean:
	$(CARGO) clean
	cd $(FRONTEND_DIR) && rm -rf dist
