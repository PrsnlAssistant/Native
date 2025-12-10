//! Behavior-based tests for PrsnlAssistant Native App
//!
//! These tests verify the core functionality of the application
//! using a BDD (Behavior-Driven Development) approach.

use prsnl_assistant::shared::{
    Message, MessageSender, MessageStatus, ImageData, Conversation, ConnectionStatus,
};
use prsnl_assistant::features::media::SelectedMedia;
use prsnl_assistant::features::conversations::ViewState;

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

mod view_state_behavior {
    use super::*;

    #[test]
    fn given_view_states_when_compared_then_work_correctly() {
        // Given: Various view states
        let list = ViewState::ConversationList;
        let chat1 = ViewState::Chat("conv-1".to_string());
        let chat2 = ViewState::Chat("conv-1".to_string());
        let chat3 = ViewState::Chat("conv-2".to_string());

        // When/Then: States should be distinguishable
        assert_eq!(list, ViewState::ConversationList);
        assert_eq!(chat1, chat2);
        assert_ne!(chat1, chat3);
        assert_ne!(list, chat1);
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
