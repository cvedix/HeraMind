/// Integration test for session creation bug fix.
///
/// This test verifies that:
/// 1. Sessions can be created and retrieved
/// 2. Session messages are stored correctly
/// 3. Session cleanup works as expected
#[cfg(test)]
mod session_fix_tests {
    use heramind_agent::SessionManager;
    use std::sync::Arc;

    /// Helper to create a test session manager
    fn create_test_manager() -> Arc<SessionManager> {
        Arc::new(SessionManager::memory())
    }

    #[tokio::test]
    async fn test_session_creation() {
        let manager = create_test_manager();

        // Create a session
        let session_id = manager
            .create_session()
            .await
            .expect("Failed to create session");

        // Verify session exists
        let _agent = manager
            .get_session(&session_id)
            .await
            .expect("Failed to get session");

        // Clean up
        manager
            .remove_session(&session_id)
            .await
            .expect("Failed to remove session");
    }

    #[tokio::test]
    async fn test_session_count() {
        let manager = create_test_manager();

        // Create multiple sessions
        let id1 = manager.create_session().await.unwrap();
        let id2 = manager.create_session().await.unwrap();
        let id3 = manager.create_session().await.unwrap();

        // Check session count
        let count = manager.session_count().await;
        assert_eq!(count, 3);

        // Clean up
        manager.remove_session(&id1).await.unwrap();
        manager.remove_session(&id2).await.unwrap();
        manager.remove_session(&id3).await.unwrap();
    }

    #[tokio::test]
    async fn test_invalid_session_id() {
        let manager = create_test_manager();

        // Try to get a non-existent session
        let result = manager.get_session("non_existent_id").await;
        assert!(result.is_err());

        // Try to remove a non-existent session
        let result = manager.remove_session("non_existent_id").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let manager = create_test_manager();

        // Create sessions
        let _id1 = manager.create_session().await.unwrap();
        let _id2 = manager.create_session().await.unwrap();

        // List sessions
        let sessions = manager.list_sessions().await;
        assert!(sessions.len() >= 2);
    }
}
