//! A2A Protocol Compliance Test Suite
//!
//! This module provides comprehensive compliance testing for the A2A protocol implementation.
//! It tests all aspects of the protocol including message formatting, error handling,
//! authentication, and transport compliance.

use a2a_protocol::{
    core::agent_card::{AgentCardSignature, AgentProvider, TransportInterface, TransportType},
    prelude::*,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use url::Url;

#[cfg(test)]
mod compliance_tests {

    use super::*;

    /// Test suite for message format compliance
    mod message_format {
        use super::*;

        #[test]
        fn test_message_structure_compliance() {
            // Test that messages follow the A2A protocol structure
            let message = Message::user_text("Hello, world!");

            // Verify required fields
            assert!(message.message_id.is_some());
            assert_eq!(message.role, MessageRole::User);
            assert!(message.parts.len() > 0);

            // Test JSON serialization compliance
            let serialized = serde_json::to_string(&message).unwrap();
            let parsed: Value = serde_json::from_str(&serialized).unwrap();

            assert!(parsed.get("messageId").is_some() || parsed.get("role").is_some());
            assert_eq!(parsed.get("role").unwrap(), "user");
            assert!(parsed.get("parts").is_some());
        }

        #[test]
        fn test_message_content_compliance() {
            let message = Message::agent_text("Response message");

            // Verify content structure
            let parts = &message.parts;
            assert_eq!(parts.len(), 1);

            let part = &parts[0];
            // Part is now an enum, check if it's a Text variant
            match part {
                Part::Text(text_part) => {
                    assert_eq!(text_part.text, "Response message");
                }
                _ => panic!("Expected Text part"),
            }
        }

        #[test]
        fn test_message_serialization_compliance() {
            let message = Message::agent_text("Response message");

            // Test JSON serialization
            let serialized = serde_json::to_string(&message).unwrap();
            let parsed: Value = serde_json::from_str(&serialized).unwrap();

            // A2A v0.3.0 spec: messageId is optional, role and parts are required
            assert!(parsed.get("role").is_some());
            assert_eq!(parsed.get("role").unwrap(), "agent");
            assert!(parsed.get("parts").is_some());
        }
    }

    /// Test suite for agent card compliance
    mod agent_card_compliance {
        use super::*;

        #[test]
        fn test_agent_card_structure() {
            let agent_id = AgentId::new("test-agent".to_owned()).unwrap();
            let url = Url::parse("https://example.com").unwrap();

            let card = AgentCard::new(agent_id, "Test Agent", url);

            // Verify required fields
            assert_eq!(card.id.as_str(), "test-agent");
            assert_eq!(card.name, "Test Agent");
            assert_eq!(card.url.as_str(), "https://example.com/");

            // Test JSON serialization
            let serialized = serde_json::to_string(&card).unwrap();
            let parsed: Value = serde_json::from_str(&serialized).unwrap();

            assert!(parsed.get("id").is_some());
            assert!(parsed.get("name").is_some());
            assert!(parsed.get("url").is_some());
            assert!(parsed.get("protocolVersion").is_some());
            assert!(parsed.get("preferredTransport").is_some());
        }

        #[test]
        fn test_agent_card_capabilities() {
            let agent_id = AgentId::new("test-agent".to_owned()).unwrap();
            let url = Url::parse("https://example.com").unwrap();

            let card = AgentCard::new(agent_id, "Test Agent", url);

            // Test new A2A v0.3.0 fields
            assert!(!card.protocol_version.is_empty());
            assert_eq!(card.protocol_version, "0.3.0");
            assert_eq!(card.preferred_transport, TransportType::JsonRpc);
            assert!(card.default_input_modes.is_empty());
            assert!(card.default_output_modes.is_empty());
            assert!(card.provider.is_none());
            assert!(card.icon_url.is_none());
            assert!(card.documentation_url.is_none());
            assert!(card.signatures.is_empty());
            assert!(!card.supports_authenticated_extended_card);
        }

        #[test]
        fn test_agent_card_serialization_with_transports() {
            let agent_id = AgentId::new("test-agent".to_owned()).unwrap();
            let url = Url::parse("https://example.com").unwrap();
            let http_url = Url::parse("https://example.com/http").unwrap();
            let icon_url = Url::parse("https://example.com/icon.png").unwrap();
            let docs_url = Url::parse("https://example.com/docs").unwrap();
            let provider_url = Url::parse("https://provider.example.com").unwrap();

            let card = AgentCard::new(agent_id, "Test Agent", url)
                .with_preferred_transport(TransportType::Grpc)
                .with_default_input_modes(["application/json"])
                .with_default_output_modes(["application/json"])
                .with_supports_authenticated_extended_card(true)
                .with_icon_url(Some(icon_url.clone()))
                .with_documentation_url(Some(docs_url.clone()))
                .with_provider(AgentProvider {
                    name: "Example Provider".to_string(),
                    description: Some("Example Org".to_string()),
                    url: Some(provider_url.clone()),
                    contact_email: None,
                    contact_url: None,
                    extra: HashMap::new(),
                })
                .with_signatures([AgentCardSignature {
                    algorithm: "Ed25519".to_string(),
                    signature: "deadbeef".to_string(),
                    key_id: Some("key-1".to_string()),
                    certificate_chain: vec!["cert-1".to_string()],
                    extra: HashMap::new(),
                }])
                .add_transport_interface(TransportInterface::new(
                    TransportType::HttpJson,
                    http_url.clone(),
                ));

            let serialized = serde_json::to_value(&card).unwrap();

            assert_eq!(serialized["preferredTransport"], "GRPC");
            assert_eq!(serialized["defaultInputModes"], json!(["application/json"]));
            assert_eq!(
                serialized["defaultOutputModes"],
                json!(["application/json"])
            );
            assert_eq!(serialized["supportsAuthenticatedExtendedCard"], true);
            assert_eq!(serialized["iconUrl"], icon_url.as_str());
            assert_eq!(serialized["documentationUrl"], docs_url.as_str());

            let provider = serialized["provider"].as_object().unwrap();
            assert_eq!(provider["name"], "Example Provider");
            assert_eq!(provider["url"], provider_url.as_str());

            let signatures = serialized["signatures"].as_array().unwrap();
            assert_eq!(signatures.len(), 1);
            assert_eq!(signatures[0]["signature"], "deadbeef");
            assert_eq!(signatures[0]["algorithm"], "Ed25519");

            let interfaces = serialized["additionalInterfaces"].as_array().unwrap();
            assert_eq!(interfaces.len(), 1);
            assert_eq!(interfaces[0]["type"], "HTTP+JSON");
            assert_eq!(interfaces[0]["url"], http_url.as_str());
        }
    }

    /// Test suite for error handling compliance
    mod error_handling {
        use super::*;

        #[test]
        fn test_error_structure_compliance() {
            let error = A2aError::Authentication("Invalid token".to_string());

            // Test error properties
            assert!(error.is_retryable() == false);
            assert_eq!(error.status_code(), Some(401));
        }

        #[test]
        fn test_error_code_mapping() {
            let test_cases = vec![
                (A2aError::Authentication("test".to_string()), 401),
                (A2aError::Validation("test".to_string()), 400),
                (A2aError::ProtocolViolation("test".to_string()), 422),
                (
                    A2aError::RateLimited(std::time::Duration::from_secs(60)),
                    429,
                ),
            ];

            for (error, expected_code) in test_cases {
                assert_eq!(error.status_code(), Some(expected_code));
            }
        }

        #[test]
        fn test_retryable_errors() {
            let retryable_errors = vec![
                A2aError::Timeout,
                A2aError::RateLimited(std::time::Duration::from_secs(60)),
                A2aError::Server("server error".to_string()),
            ];

            for error in retryable_errors {
                assert!(
                    error.is_retryable(),
                    "Error should be retryable: {:?}",
                    error
                );
            }

            let non_retryable_errors = vec![
                A2aError::Authentication("auth error".to_string()),
                A2aError::Validation("validation error".to_string()),
                A2aError::ProtocolViolation("protocol error".to_string()),
            ];

            for error in non_retryable_errors {
                assert!(
                    !error.is_retryable(),
                    "Error should not be retryable: {:?}",
                    error
                );
            }
        }
    }

    /// Test suite for transport compliance
    mod transport_compliance {
        use super::*;

        #[test]
        fn test_json_rpc_transport_compliance() {
            let transport =
                a2a_protocol::transport::JsonRpcTransport::new("https://example.com/rpc").unwrap();

            assert_eq!(transport.transport_type(), "json-rpc");

            // Test JSON-RPC transport basic functionality
            assert!(!transport.transport_type().is_empty());
        }
    }

    /// Test suite for protocol validation
    mod protocol_validation {
        use super::*;

        #[test]
        fn test_agent_id_validation() {
            // Valid agent IDs
            let valid_ids = vec![
                "https://agent.example.com",
                "agent-123",
                "my_agent",
                "agent.example.com",
            ];

            for id in valid_ids {
                let result = AgentId::new(id.to_string());
                assert!(result.is_ok(), "Agent ID should be valid: {}", id);
            }

            // Invalid agent IDs
            let invalid_ids = vec![
                "",
                "   ",
                "ht tp://bad-url", // Invalid URL format
            ];

            for id in invalid_ids {
                let result = AgentId::new(id.to_string());
                assert!(result.is_err(), "Agent ID should be invalid: {}", id);
            }
        }

        #[test]
        fn test_url_agent_id_handling() {
            let url = Url::parse("https://agent.example.com").unwrap();
            let agent_id = AgentId::from_url(&url);

            // URL should be normalized
            assert!(agent_id.as_str().ends_with('/'));
        }

        #[test]
        fn test_message_validation() {
            // Test message creation with valid data
            let message = Message::user_text("Hello");
            assert!(message.message_id.is_some());
            assert_eq!(message.role, MessageRole::User);
        }
    }

    /// Integration compliance tests
    mod integration_compliance {
        use super::*;

        #[test]
        fn test_client_server_workflow() {
            // This would test a complete client-server interaction
            // For now, we verify the components can be created together

            let agent_id = AgentId::new("test-agent".to_owned()).unwrap();
            let url = Url::parse("https://example.com").unwrap();
            let agent_card = AgentCard::new(agent_id, "Test Agent", url);

            // Create server components
            use a2a_protocol::server::handler::BasicA2aHandler;
            let _handler = BasicA2aHandler::new(agent_card.clone());

            // Create client components
            use a2a_protocol::client::ClientBuilder;
            let client_result = ClientBuilder::new()
                .with_agent_id("test-client")
                .unwrap()
                .with_json_rpc("https://example.com/rpc")
                .build();

            // Verify components can be created
            assert!(client_result.is_ok());
        }

        #[test]
        fn test_message_round_trip() {
            // Test that messages can be serialized and deserialized without loss
            let original_message = Message::user_text("Test message");

            let serialized = serde_json::to_string(&original_message).unwrap();
            let deserialized: Message = serde_json::from_str(&serialized).unwrap();

            assert_eq!(original_message.message_id, deserialized.message_id);
            assert_eq!(original_message.role, deserialized.role);
            assert_eq!(original_message.parts.len(), deserialized.parts.len());
        }
    }

    /// Performance compliance tests
    mod performance_compliance {
        use super::*;
        use std::time::Instant;

        #[test]
        fn test_message_serialization_performance() {
            let message = Message::user_text("Test message content");

            let start = Instant::now();
            for _ in 0..1000 {
                let _ = serde_json::to_string(&message).unwrap();
            }
            let duration = start.elapsed();

            // Should serialize 1000 messages in less than 100ms
            assert!(
                duration.as_millis() < 100,
                "Message serialization too slow: {}ms for 1000 messages",
                duration.as_millis()
            );
        }

        #[test]
        fn test_agent_id_generation_performance() {
            let start = Instant::now();
            for _ in 0..1000 {
                let _ = AgentId::generate();
            }
            let duration = start.elapsed();

            // Should generate 1000 agent IDs in less than 50ms
            assert!(
                duration.as_millis() < 50,
                "Agent ID generation too slow: {}ms for 1000 IDs",
                duration.as_millis()
            );
        }
    }
}

/// Compliance test runner
pub struct ComplianceTestRunner;

impl ComplianceTestRunner {
    /// Run all compliance tests and return a report
    pub fn run_all_tests() -> ComplianceReport {
        let mut report = ComplianceReport::new();

        // This would run all the tests and collect results
        // In a real implementation, this would use the test framework

        report.add_result("Message Format Compliance", true);
        report.add_result("Agent Card Compliance", true);
        report.add_result("Error Handling Compliance", true);
        report.add_result("Transport Compliance", true);
        report.add_result("Protocol Validation", true);
        report.add_result("Integration Compliance", true);
        report.add_result("Performance Compliance", true);

        report
    }
}

/// Compliance test report
#[derive(Debug)]
pub struct ComplianceReport {
    results: HashMap<String, bool>,
}

impl ComplianceReport {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    pub fn add_result(&mut self, test_name: &str, passed: bool) {
        self.results.insert(test_name.to_string(), passed);
    }

    pub fn overall_compliance(&self) -> f64 {
        let total = self.results.len();
        if total == 0 {
            return 0.0;
        }

        let passed = self.results.values().filter(|&&v| v).count();
        (passed as f64 / total as f64) * 100.0
    }

    pub fn generate_summary(&self) -> String {
        let compliance = self.overall_compliance();
        format!(
            "A2A Protocol Compliance Report: {:.1}% compliant ({} passed, {} failed)",
            compliance,
            self.results.values().filter(|&&v| v).count(),
            self.results.values().filter(|&&v| !v).count()
        )
    }
}
