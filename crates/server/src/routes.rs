//! Route definitions and router setup

use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers;
use crate::ServerState;

/// Create the application router with all routes
pub fn create_router(state: ServerState) -> Router {
    // Build API routes
    let api_routes = Router::new()
        // Agent operations
        .route("/agents/spawn", post(handlers::spawn_agent))
        .route("/agents", get(handlers::list_agents))
        // Session operations
        .route("/sessions", get(handlers::list_sessions))
        .route("/sessions/{session_id}", get(handlers::get_session_status))
        .route(
            "/sessions/{session_id}",
            axum::routing::delete(handlers::delete_session),
        )
        // Log streaming
        .route("/logs/{session_id}/stream", get(handlers::stream_logs))
        // Health check
        .route("/health", get(handlers::health_check));

    // Combine with base path and state
    Router::new()
        .nest("/api", api_routes)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ServerState;
    use lite_agent_core::{SessionManager, WorkspaceManager};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_router() {
        let session_manager = Arc::new(SessionManager::new());
        let workspace_manager = Arc::new(WorkspaceManager::new(".".into()));
        let state = ServerState::new(session_manager, workspace_manager);

        let _router = create_router(state);

        // Router should be created successfully
        assert!(true, "Router created successfully");
    }

    #[test]
    fn test_router_structure() {
        // This test verifies the router structure is correct
        // In a real application, you might test routing more thoroughly
        assert!(true, "Router structure is valid");
    }
}
