##@ Build

.PHONY: help
help: ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

.PHONY: generate-frontend-client
generate-frontend-client: ## Generate client code for frontend
	openapi-generator-cli generate -g rust -i /local/openapi.yaml -o /local/frontend/client

.PHONY: build-frontend
build-frontend: trunk ## Build frontend
	cd frontend && RUSTFLAGS='--cfg getrandom_backend="wasm_js"' trunk build

.PHONY: build-backend
build-backend: ## Build backend
	cd backend && cargo build

##@ Tools

.PHONY: trunk
trunk: ## install trunk
	cargo install trunk
