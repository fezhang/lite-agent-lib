//! HTTP request handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{sse::Event, IntoResponse, Json, Sse},
};
use futures_util::StreamExt;
use serde_json::json;

use crate::api::{
    ErrorResponse, ListAgentsResponse, ListSessionsResponse, SessionStatusResponse,
    SpawnRequest, SpawnResponse,
};
use crate::ServerState;
use lite_agent_core::{
    AgentError, ExecutionStatus, SessionStatus,
};

/// Spawn a new agent or continue an existing session
pub async fn spawn_agent(
    State(state): State<ServerState>,
    Json(request): Json<SpawnRequest>,
) -> impl IntoResponse {
    // Get the agent executor
    let agent = match state.agent_registry.get(&request.agent_type).await {
        Some(a) => a,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                &format!("Agent type '{}' not found", request.agent_type),
                None,
            )
        }
    };

    // Convert config options to AgentConfig
    let config: lite_agent_core::AgentConfig = request.config.clone().into();

    // Create or continue session
    let session_id = if let Some(sid) = request.session_id {
        // Continue existing session
        if state.session_manager.get_session(&sid).await.is_none() {
            return error_response(
                StatusCode::NOT_FOUND,
                &format!("Session '{}' not found", sid),
                None,
            );
        }

        // Add new execution
        let execution_id = match state
            .session_manager
            .add_execution(&sid, request.input.clone())
            .await
        {
            Ok(id) => id,
            Err(e) => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to create execution",
                    Some(format!("{:?}", e)),
                )
            }
        };

        // Spawn the agent with follow-up
        match agent.spawn_follow_up(&config, &request.input, &sid).await {
            Ok(_spawned) => {
                // Update session status
                let _ = state
                    .session_manager
                    .update_session_status(&sid, SessionStatus::Active)
                    .await;
            }
            Err(e) => {
                let _ = state
                    .session_manager
                    .update_execution(&sid, &execution_id, ExecutionStatus::Failed, None)
                    .await;
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to spawn agent",
                    Some(format!("{:?}", e)),
                );
            }
        }

        sid
    } else {
        // Create new session
        let session_id = match state
            .session_manager
            .create_session(request.agent_type.clone(), request.input.clone())
            .await
        {
            Ok(id) => id,
            Err(e) => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to create session",
                    Some(format!("{:?}", e)),
                )
            }
        };

        // Get the execution ID from the session
        let execution_id = match state.session_manager.get_session(&session_id).await {
            Some(session) => {
                if let Some(execution) = session.executions.last() {
                    execution.id.clone()
                } else {
                    return error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to get execution ID",
                        None,
                    );
                }
            }
            None => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to get session",
                    None,
                )
            }
        };

        // Spawn the agent
        match agent.spawn(&config, &request.input).await {
            Ok(_spawned) => {
                // Session status already set to Active by create_session
            }
            Err(e) => {
                let _ = state
                    .session_manager
                    .update_execution(&session_id, &execution_id, ExecutionStatus::Failed, None)
                    .await;
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to spawn agent",
                    Some(format!("{:?}", e)),
                );
            }
        }

        session_id
    };

    let response = SpawnResponse {
        session_id: session_id.clone(),
        execution_id: {
            let session = state.session_manager.get_session(&session_id).await.unwrap();
            session.executions.last().unwrap().id.clone()
        },
        agent_type: request.agent_type,
        status: "started".to_string(),
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

/// Get session status
pub async fn get_session_status(
    State(state): State<ServerState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let session = match state.session_manager.get_session(&session_id).await {
        Some(s) => s,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                &format!("Session '{}' not found", session_id),
                None,
            )
        }
    };

    let response = SessionStatusResponse {
        session_id: session.id.clone(),
        agent_type: session.agent_type.clone(),
        status: format!("{:?}", session.status),
        execution_count: session.executions.len(),
        created_at: session.created_at.to_rfc3339(),
        updated_at: session.updated_at.to_rfc3339(),
    };

    Json(response).into_response()
}

/// List all sessions
pub async fn list_sessions(State(state): State<ServerState>) -> impl IntoResponse {
    let sessions = state.session_manager.get_all_sessions().await;

    let session_responses: Vec<SessionStatusResponse> = sessions
        .into_iter()
        .map(|s| SessionStatusResponse {
            session_id: s.id.clone(),
            agent_type: s.agent_type.clone(),
            status: format!("{:?}", s.status),
            execution_count: s.executions.len(),
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        })
        .collect();

    let response = ListSessionsResponse {
        total: session_responses.len(),
        sessions: session_responses,
    };

    Json(response).into_response()
}

/// Delete a session
pub async fn delete_session(
    State(state): State<ServerState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.session_manager.delete_session(&session_id).await {
        Ok(_) => (
            StatusCode::NO_CONTENT,
            Json(json!({"message": "Session deleted successfully"})),
        )
            .into_response(),
        Err(AgentError::SessionNotFound(_)) => error_response(
            StatusCode::NOT_FOUND,
            &format!("Session '{}' not found", session_id),
            None,
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete session",
            Some(format!("{:?}", e)),
        ),
    }
}

/// List available agents
pub async fn list_agents(State(state): State<ServerState>) -> impl IntoResponse {
    let agents = state.agent_registry.get_all_info().await;

    let response = ListAgentsResponse {
        total: agents.len(),
        agents,
    };

    Json(response).into_response()
}

/// Stream logs for a session via Server-Sent Events (SSE)
pub async fn stream_logs(
    State(state): State<ServerState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    // Check if session exists
    let session_exists = state.session_manager.get_log_store(&session_id).await.is_some();

    // Create SSE stream
    let stream = async_stream::stream! {
        if !session_exists {
            // Send error event
            yield Ok::<_, axum::Error>(Event::default()
                .comment("Session not found"));
        } else {
            // Send initial event
            yield Ok::<_, axum::Error>(Event::default()
                .json_data(json!({"type": "stream_started", "session_id": session_id}))
                .unwrap());

            // Get log store and subscribe
            if let Some(log_store) = state.session_manager.get_log_store(&session_id).await {
                let mut log_stream = log_store.subscribe();

                // Stream log entries
                while let Some(entry) = log_stream.next().await {
                    let event = Event::default()
                        .json_data(entry)
                        .unwrap_or_else(|_| Event::default().data("error serializing log entry"));
                    yield Ok::<_, axum::Error>(event);
                }
            }

            // Send end event
            yield Ok::<_, axum::Error>(Event::default()
                .json_data(json!({"type": "stream_ended"}))
                .unwrap());
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(1))
            .text("keepalive"),
    )
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }))
}

/// Helper function to create error responses
fn error_response(
    status: StatusCode,
    message: &str,
    details: Option<String>,
) -> axum::response::Response {
    let error_response = ErrorResponse {
        error: message.to_string(),
        details,
    };

    (status, Json(error_response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::AgentConfigOptions;

    #[tokio::test]
    async fn test_spawn_agent_request() {
        let request = SpawnRequest {
            agent_type: "echo".to_string(),
            input: "test".to_string(),
            session_id: None,
            config: AgentConfigOptions::default(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: SpawnRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.agent_type, "echo");
    }

    #[tokio::test]
    async fn test_error_response() {
        let response = error_response(StatusCode::BAD_REQUEST, "test error", Some("details".to_string()));
        let status = response.status();

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        let json = response.into_response();

        assert!(json.status().is_success());
    }
}
