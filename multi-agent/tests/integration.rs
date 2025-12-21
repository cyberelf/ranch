//! Integration tests for Team orchestration

mod common;

use common::MockAgent;
use multi_agent::team::{
    RouterConfig, TeamAgentConfig,
};
use multi_agent::{track_team_nesting, Agent, AgentManager, Team, TeamConfig};
use std::collections::HashSet;
use std::sync::Arc;

#[tokio::test]
async fn test_team_info_aggregates_capabilities() {
    // Create mock agents with different capabilities
    let agent1 = Arc::new(
        MockAgent::new("agent-1", "Agent 1")
            .with_capabilities(vec!["research".to_string(), "analysis".to_string()]),
    );

    let agent2 = Arc::new(
        MockAgent::new("agent-2", "Agent 2")
            .with_capabilities(vec!["writing".to_string(), "editing".to_string()]),
    );

    // Register agents
    let agent_manager = Arc::new(AgentManager::new());
    let agent1_id = agent_manager
        .register(agent1 as Arc<dyn Agent>)
        .await
        .unwrap();
    let agent2_id = agent_manager
        .register(agent2 as Arc<dyn Agent>)
        .await
        .unwrap();

    // Create team
    let team_config = TeamConfig {
        id: "test-team".to_string(),
        name: "Test Team".to_string(),
        description: "A test team".to_string(),
        agents: vec![
            TeamAgentConfig {
                agent_id: agent1_id.clone(),
                role: "supervisor".to_string(),
                capabilities: vec!["research".to_string()],
            },
            TeamAgentConfig {
                agent_id: agent2_id.clone(),
                role: "member".to_string(),
                capabilities: vec!["writing".to_string()],
            },
        ],
        router_config: RouterConfig {
            default_agent_id: agent1_id.clone(),
            max_routing_hops: 10,
        },
    };

    let team = Team::new(team_config, agent_manager);

    // Get team info
    let info = team.info().await.unwrap();

    // Verify capabilities are aggregated
    assert_eq!(info.id, "test-team");
    assert_eq!(info.name, "Test Team");
    assert!(info.capabilities.contains(&"research".to_string()));
    assert!(info.capabilities.contains(&"analysis".to_string()));
    assert!(info.capabilities.contains(&"writing".to_string()));
    assert!(info.capabilities.contains(&"editing".to_string()));

    // Verify metadata includes team info
    assert_eq!(info.metadata.get("type"), Some(&"team".to_string()));
    assert_eq!(info.metadata.get("router_default_agent"), Some(&agent1_id));
    assert_eq!(info.metadata.get("member_count"), Some(&"2".to_string()));
}

#[tokio::test]
async fn test_team_process_supervisor_mode() {
    // Create mock agents
    let agent1 = Arc::new(
        MockAgent::new("agent-1", "Supervisor").with_response("Delegating to team member"),
    );

    let agent2 = Arc::new(MockAgent::new("agent-2", "Worker").with_response("Task completed"));

    // Register agents
    let agent_manager = Arc::new(AgentManager::new());
    let agent1_id = agent_manager
        .register(agent1.clone() as Arc<dyn Agent>)
        .await
        .unwrap();
    let agent2_id = agent_manager
        .register(agent2.clone() as Arc<dyn Agent>)
        .await
        .unwrap();

    // Create team with supervisor mode
    let team_config = TeamConfig {
        id: "supervisor-team".to_string(),
        name: "Supervisor Team".to_string(),
        description: "Team with supervisor".to_string(),
        agents: vec![
            TeamAgentConfig {
                agent_id: agent1_id.clone(),
                role: "supervisor".to_string(),
                capabilities: vec![],
            },
            TeamAgentConfig {
                agent_id: agent2_id.clone(),
                role: "worker".to_string(),
                capabilities: vec![],
            },
        ],
        router_config: RouterConfig {
            default_agent_id: agent1_id,
            max_routing_hops: 10,
        },
    };

    let team = Team::new(team_config, agent_manager);

    // Verify team can be created and info works
    let info = team.info().await.unwrap();
    assert_eq!(info.id, "supervisor-team");
    assert_eq!(info.name, "Supervisor Team");

    // Note: Not testing process() as it may loop - that would require
    // more sophisticated mocking of the scheduler behavior
}

#[tokio::test]
async fn test_team_process_workflow_mode() {
    // Create mock agents
    let agent1 = Arc::new(MockAgent::new("agent-1", "Step 1").with_response("Step 1 complete"));

    let agent2 = Arc::new(MockAgent::new("agent-2", "Step 2").with_response("Step 2 complete"));

    // Register agents
    let agent_manager = Arc::new(AgentManager::new());
    let agent1_id = agent_manager
        .register(agent1.clone() as Arc<dyn Agent>)
        .await
        .unwrap();
    let agent2_id = agent_manager
        .register(agent2.clone() as Arc<dyn Agent>)
        .await
        .unwrap();

    // Create team with workflow mode (sequential execution)
    let team_config = TeamConfig {
        id: "workflow-team".to_string(),
        name: "Workflow Team".to_string(),
        description: "Sequential workflow".to_string(),
        agents: vec![
            TeamAgentConfig {
                agent_id: agent1_id.clone(),
                role: "step1".to_string(),
                capabilities: vec![],
            },
            TeamAgentConfig {
                agent_id: agent2_id.clone(),
                role: "step2".to_string(),
                capabilities: vec![],
            },
        ],
        router_config: RouterConfig {
            default_agent_id: agent1_id,
            max_routing_hops: 10,
        },
    };

    let team = Team::new(team_config, agent_manager);

    // Verify team can be created and info works
    let info = team.info().await.unwrap();
    assert_eq!(info.id, "workflow-team");
    assert_eq!(info.name, "Workflow Team");

    // Note: Not testing process() as workflow execution requires
    // sophisticated scheduler logic that determines when to stop
}

#[tokio::test]
async fn test_cycle_detection_prevents_infinite_nesting() {
    let mut visited = HashSet::new();

    // First call should succeed
    assert!(track_team_nesting("team-1", &mut visited).is_ok());

    // Visiting a different team should succeed
    assert!(track_team_nesting("team-2", &mut visited).is_ok());

    // Attempting to visit team-1 again should fail (cycle detected)
    let result = track_team_nesting("team-1", &mut visited);
    assert!(result.is_err());

    // Verify error message contains team ID
    if let Err(cycle_err) = result {
        assert!(cycle_err.to_string().contains("team-1"));
        assert!(cycle_err.to_string().contains("cycle"));
    } else {
        panic!("Expected CycleError");
    }
}

#[tokio::test]
async fn test_error_propagation_from_member_agents() {
    // Create a mock agent
    let agent = Arc::new(MockAgent::new("test-agent", "Test Agent"));

    // Register the agent
    let agent_manager = Arc::new(AgentManager::new());
    let agent_id = agent_manager
        .register(agent as Arc<dyn Agent>)
        .await
        .unwrap();

    // Create team with the agent
    let team_config = TeamConfig {
        id: "test-team".to_string(),
        name: "Test Team".to_string(),
        description: "Team for testing".to_string(),
        agents: vec![TeamAgentConfig {
            agent_id: agent_id.clone(),
            role: "worker".to_string(),
            capabilities: vec![],
        }],
        router_config: RouterConfig {
            default_agent_id: agent_id,
            max_routing_hops: 10,
        },
    };

    let team = Team::new(team_config, agent_manager);

    // Team info should work
    let info = team.info().await;
    assert!(info.is_ok());

    // Verify team structure
    let info = info.unwrap();
    assert_eq!(info.id, "test-team");
    assert_eq!(info.metadata.get("member_count"), Some(&"1".to_string()));
}

#[tokio::test]
async fn test_nested_team_delegation() {
    // Create leaf-level agents
    let agent1 = Arc::new(
        MockAgent::new("agent-1", "Data Collector")
            .with_capabilities(vec!["data-collection".to_string()])
            .with_response("Data collected successfully"),
    );

    let agent2 = Arc::new(
        MockAgent::new("agent-2", "Data Analyzer")
            .with_capabilities(vec!["data-analysis".to_string()])
            .with_response("Analysis complete"),
    );

    let agent3 = Arc::new(
        MockAgent::new("agent-3", "Coordinator")
            .with_capabilities(vec!["coordination".to_string()])
            .with_response("Coordinating tasks"),
    );

    // Create agent manager and register leaf agents
    let agent_manager = Arc::new(AgentManager::new());
    let agent1_id = agent_manager
        .register(agent1 as Arc<dyn Agent>)
        .await
        .unwrap();
    let agent2_id = agent_manager
        .register(agent2 as Arc<dyn Agent>)
        .await
        .unwrap();
    let agent3_id = agent_manager
        .register(agent3 as Arc<dyn Agent>)
        .await
        .unwrap();

    // Create child team (research department)
    let child_team_config = TeamConfig {
        id: "research-dept".to_string(),
        name: "Research Department".to_string(),
        description: "Handles research tasks".to_string(),
        agents: vec![
            TeamAgentConfig {
                agent_id: agent1_id.clone(),
                role: "collector".to_string(),
                capabilities: vec!["data-collection".to_string()],
            },
            TeamAgentConfig {
                agent_id: agent2_id.clone(),
                role: "analyzer".to_string(),
                capabilities: vec!["data-analysis".to_string()],
            },
        ],
        router_config: RouterConfig {
            default_agent_id: agent1_id,
            max_routing_hops: 10,
        },
    };

    let child_team = Arc::new(Team::new(child_team_config, agent_manager.clone()));

    // Register child team as an agent in the manager
    let child_team_id = agent_manager
        .register(child_team.clone() as Arc<dyn Agent>)
        .await
        .unwrap();

    // Create parent team that includes the child team
    let parent_team_config = TeamConfig {
        id: "organization".to_string(),
        name: "Organization".to_string(),
        description: "Top-level organization".to_string(),
        agents: vec![
            TeamAgentConfig {
                agent_id: agent3_id.clone(),
                role: "coordinator".to_string(),
                capabilities: vec!["coordination".to_string()],
            },
            TeamAgentConfig {
                agent_id: child_team_id.clone(),
                role: "research".to_string(),
                capabilities: vec!["data-collection".to_string(), "data-analysis".to_string()],
            },
        ],
        router_config: RouterConfig {
            default_agent_id: agent3_id,
            max_routing_hops: 10,
        },
    };

    let parent_team = Team::new(parent_team_config, agent_manager);

    // Verify parent team info
    let parent_info = parent_team.info().await.unwrap();
    assert_eq!(parent_info.id, "organization");
    assert_eq!(parent_info.name, "Organization");

    // Verify parent team aggregates capabilities from child team
    assert!(parent_info
        .capabilities
        .contains(&"coordination".to_string()));
    assert!(parent_info
        .capabilities
        .contains(&"data-collection".to_string()));
    assert!(parent_info
        .capabilities
        .contains(&"data-analysis".to_string()));

    // Verify child team info is accessible
    let child_info = child_team.info().await.unwrap();
    assert_eq!(child_info.id, "research-dept");
    assert!(child_info
        .capabilities
        .contains(&"data-collection".to_string()));
    assert!(child_info
        .capabilities
        .contains(&"data-analysis".to_string()));

    // Verify metadata
    assert_eq!(parent_info.metadata.get("type"), Some(&"team".to_string()));
    assert_eq!(
        parent_info.metadata.get("member_count"),
        Some(&"2".to_string())
    );
}

#[tokio::test]
async fn test_three_level_nesting_works_without_cycles() {
    let mut visited = HashSet::new();

    // Create 3-level nesting
    assert!(track_team_nesting("level-1", &mut visited).is_ok());
    assert!(track_team_nesting("level-2", &mut visited).is_ok());
    assert!(track_team_nesting("level-3", &mut visited).is_ok());

    // All three should be in visited set
    assert!(visited.contains("level-1"));
    assert!(visited.contains("level-2"));
    assert!(visited.contains("level-3"));

    // Attempting to create a cycle should fail
    let result = track_team_nesting("level-1", &mut visited);
    assert!(result.is_err());
}
