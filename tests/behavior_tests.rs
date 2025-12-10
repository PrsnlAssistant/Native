//! Behavior-based tests for PrsnlAssistant Native App
//!
//! These tests verify the core functionality of the application
//! using a BDD (Behavior-Driven Development) approach.

use prsnl_assistant::state::*;
use prsnl_assistant::media::SelectedMedia;

mod message_behavior {
    use super::*;

    #[test]
    fn given_empty_app_when_creating_user_message_then_has_correct_properties() {
        // Given: A new user message with some text
        let text = "Hello, Assistant!".to_string();

        // When: Creating a user message
        let message = Message::new_user(text.clone());

        // Then: Message should have correct properties
        assert_eq!(message.body, text);
        assert_eq!(message.sender, MessageSender::User);
        assert_eq!(message.status, MessageStatus::Sending);
        assert!(message.image.is_none());
        assert!(!message.id.is_empty());
    }

    #[test]
    fn given_image_data_when_creating_user_message_with_image_then_includes_image() {
        // Given: Image data and message text
        let text = "Check out this image".to_string();
        let image = ImageData {
            data: "base64encodeddata".to_string(),
            mimetype: "image/png".to_string(),
        };

        // When: Creating a user message with image
        let message = Message::new_user_with_image(text.clone(), image.clone());

        // Then: Message should include the image
        assert_eq!(message.body, text);
        assert!(message.image.is_some());
        let msg_image = message.image.unwrap();
        assert_eq!(msg_image.data, image.data);
        assert_eq!(msg_image.mimetype, image.mimetype);
    }

    #[test]
    fn given_assistant_response_when_creating_message_then_has_delivered_status() {
        // Given: An assistant response
        let id = "msg-123".to_string();
        let body = "Here's my response".to_string();

        // When: Creating an assistant message
        let message = Message::new_assistant(id.clone(), body.clone(), None);

        // Then: Message should be marked as delivered
        assert_eq!(message.id, id);
        assert_eq!(message.body, body);
        assert_eq!(message.sender, MessageSender::Assistant);
        assert_eq!(message.status, MessageStatus::Delivered);
    }

    #[test]
    fn given_system_notification_when_creating_message_then_has_system_sender() {
        // Given: A system notification
        let text = "Connection established".to_string();

        // When: Creating a system message
        let message = Message::new_system(text.clone());

        // Then: Message should have system sender
        assert_eq!(message.body, text);
        assert_eq!(message.sender, MessageSender::System);
        assert_eq!(message.status, MessageStatus::Delivered);
    }
}

mod conversation_behavior {
    use super::*;

    #[test]
    fn given_new_conversation_when_created_then_has_default_title() {
        // Given/When: Creating a new conversation without title
        let conversation = Conversation::new("conv-123".to_string(), None);

        // Then: Should have default title
        assert_eq!(conversation.id, "conv-123");
        assert_eq!(conversation.title, "New Chat");
        assert!(conversation.messages.is_empty());
        assert_eq!(conversation.message_count, 0);
    }

    #[test]
    fn given_conversation_when_adding_user_message_then_updates_metadata() {
        // Given: A conversation
        let mut conversation = Conversation::new("conv-123".to_string(), Some("Test Chat".to_string()));
        let message = Message::new_user("Hello!".to_string());
        let msg_id = message.id.clone();

        // When: Adding a user message
        conversation.add_user_message(message);

        // Then: Conversation metadata should be updated
        assert_eq!(conversation.messages.len(), 1);
        assert_eq!(conversation.message_count, 1);
        assert_eq!(conversation.last_message_preview, Some("Hello!".to_string()));
        assert!(conversation.pending_messages.contains(&msg_id));
    }

    #[test]
    fn given_pending_message_when_response_received_then_removes_from_pending() {
        // Given: A conversation with a pending message
        let mut conversation = Conversation::new("conv-123".to_string(), None);
        let user_msg = Message::new_user("Hello!".to_string());
        let user_msg_id = user_msg.id.clone();
        conversation.add_user_message(user_msg);

        assert!(conversation.pending_messages.contains(&user_msg_id));

        // When: Response is received
        let response = Message::new_assistant("resp-1".to_string(), "Hi there!".to_string(), None);
        conversation.add_response(&user_msg_id, response);

        // Then: Message should no longer be pending
        assert!(!conversation.pending_messages.contains(&user_msg_id));
        assert_eq!(conversation.messages.len(), 2);
    }

    #[test]
    fn given_message_when_error_occurs_then_marks_as_error() {
        // Given: A conversation with a message
        let mut conversation = Conversation::new("conv-123".to_string(), None);
        let message = Message::new_user("Hello!".to_string());
        let msg_id = message.id.clone();
        conversation.add_user_message(message);

        // When: An error occurs
        conversation.mark_message_error(&msg_id, "Connection failed".to_string());

        // Then: Message should be marked with error
        let msg = conversation.messages.iter().find(|m| m.id == msg_id).unwrap();
        assert!(matches!(msg.status, MessageStatus::Error(_)));
        assert!(!conversation.pending_messages.contains(&msg_id));
    }

    #[test]
    fn given_server_data_when_creating_conversation_then_parses_correctly() {
        // Given: Server data for a conversation
        let id = "native-abc12345-def6-7890".to_string();
        let last_message = Some("Last message preview".to_string());
        let last_time = Some(1699999999000i64);
        let count = 5;

        // When: Creating conversation from server data
        let conversation = Conversation::from_server(id.clone(), last_message.clone(), last_time, count);

        // Then: Should parse correctly
        assert_eq!(conversation.id, id);
        assert!(conversation.title.starts_with("Chat "));
        assert_eq!(conversation.last_message_preview, last_message);
        assert_eq!(conversation.message_count, count);
    }
}

mod app_state_behavior {
    use super::*;

    #[test]
    fn given_new_app_state_when_initialized_then_starts_at_conversation_list() {
        // Given/When: Creating new app state
        let state = AppState::new();

        // Then: Should start at conversation list view
        assert!(matches!(state.view, ViewState::ConversationList));
        assert!(state.conversations.is_empty());
        assert!(matches!(state.connection_status, ConnectionStatus::Connecting));
    }

    #[test]
    fn given_conversation_when_opening_then_switches_view() {
        // Given: App state with a conversation
        let mut state = AppState::new();
        let conv = Conversation::new("conv-123".to_string(), None);
        state.upsert_conversation(conv);

        // When: Opening the conversation
        state.open_conversation("conv-123");

        // Then: View should switch to chat
        assert!(matches!(state.view, ViewState::Chat(ref id) if id == "conv-123"));
    }

    #[test]
    fn given_chat_view_when_going_back_then_returns_to_list() {
        // Given: App state in chat view
        let mut state = AppState::new();
        let conv = Conversation::new("conv-123".to_string(), None);
        state.upsert_conversation(conv);
        state.open_conversation("conv-123");

        // When: Going back
        state.go_to_list();

        // Then: View should return to conversation list
        assert!(matches!(state.view, ViewState::ConversationList));
    }

    #[test]
    fn given_conversations_when_sorted_then_most_recent_first() {
        // Given: App state with multiple conversations
        let mut state = AppState::new();

        let mut conv1 = Conversation::new("conv-1".to_string(), None);
        let mut conv2 = Conversation::new("conv-2".to_string(), None);

        // Add messages at different times
        let msg1 = Message::new_user("First".to_string());
        conv1.add_user_message(msg1);

        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));

        let msg2 = Message::new_user("Second".to_string());
        conv2.add_user_message(msg2);

        state.upsert_conversation(conv1);
        state.upsert_conversation(conv2);

        // When: Getting sorted conversations
        let sorted = state.sorted_conversations();

        // Then: conv2 should be first (more recent)
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].id, "conv-2");
        assert_eq!(sorted[1].id, "conv-1");
    }

    #[test]
    fn given_conversation_when_deleted_then_removed_and_view_changes() {
        // Given: App state viewing a conversation
        let mut state = AppState::new();
        let conv = Conversation::new("conv-123".to_string(), None);
        state.upsert_conversation(conv);
        state.open_conversation("conv-123");

        // When: Deleting the conversation
        state.delete_conversation("conv-123");

        // Then: Conversation should be removed and view should change
        assert!(!state.conversations.contains_key("conv-123"));
        assert!(matches!(state.view, ViewState::ConversationList));
    }

    #[test]
    fn given_state_when_creating_conversation_then_switches_to_it() {
        // Given: Empty app state
        let mut state = AppState::new();

        // When: Creating a new conversation
        state.create_conversation("conv-new".to_string(), Some("My New Chat".to_string()));

        // Then: Should create and switch to the conversation
        assert!(state.conversations.contains_key("conv-new"));
        assert!(matches!(state.view, ViewState::Chat(ref id) if id == "conv-new"));
        assert_eq!(state.conversations.get("conv-new").unwrap().title, "My New Chat");
    }
}

mod media_behavior {
    use super::*;

    #[test]
    fn given_selected_media_when_cloned_then_maintains_all_fields() {
        // Given: A selected media item
        let media = SelectedMedia {
            data: "base64data".to_string(),
            mimetype: "image/jpeg".to_string(),
            filename: "photo.jpg".to_string(),
        };

        // When: Cloning the media
        let cloned = media.clone();

        // Then: All fields should be preserved
        assert_eq!(cloned.data, media.data);
        assert_eq!(cloned.mimetype, media.mimetype);
        assert_eq!(cloned.filename, media.filename);
    }

    #[test]
    fn given_two_identical_media_when_compared_then_are_equal() {
        // Given: Two identical media items
        let media1 = SelectedMedia {
            data: "base64data".to_string(),
            mimetype: "image/png".to_string(),
            filename: "image.png".to_string(),
        };
        let media2 = SelectedMedia {
            data: "base64data".to_string(),
            mimetype: "image/png".to_string(),
            filename: "image.png".to_string(),
        };

        // When/Then: They should be equal
        assert_eq!(media1, media2);
    }

    #[test]
    fn given_different_media_when_compared_then_are_not_equal() {
        // Given: Two different media items
        let media1 = SelectedMedia {
            data: "base64data1".to_string(),
            mimetype: "image/png".to_string(),
            filename: "image1.png".to_string(),
        };
        let media2 = SelectedMedia {
            data: "base64data2".to_string(),
            mimetype: "image/jpeg".to_string(),
            filename: "image2.jpg".to_string(),
        };

        // When/Then: They should not be equal
        assert_ne!(media1, media2);
    }
}

mod connection_status_behavior {
    use super::*;

    #[test]
    fn given_connection_states_when_compared_then_work_correctly() {
        // Given: Various connection states
        let connecting = ConnectionStatus::Connecting;
        let connected = ConnectionStatus::Connected;
        let disconnected = ConnectionStatus::Disconnected;
        let reconnecting = ConnectionStatus::Reconnecting;

        // When/Then: States should be distinguishable
        assert_eq!(connecting, ConnectionStatus::Connecting);
        assert_eq!(connected, ConnectionStatus::Connected);
        assert_eq!(disconnected, ConnectionStatus::Disconnected);
        assert_eq!(reconnecting, ConnectionStatus::Reconnecting);

        assert_ne!(connecting, connected);
        assert_ne!(connected, disconnected);
    }
}
