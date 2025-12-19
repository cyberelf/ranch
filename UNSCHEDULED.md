The unscheduled requirements
- Protocol enhancement
  - add a server in a2a protocol for hosting of multiple agents

- Multi-agent enhancement
  - Refactor the Agent scheduler into actor model, each agent is an actor of `ractor`, and the scheduler is the orchestration system instead of simple routing decision maker. Also we need to reimplement the orchestration patterns, starting from:
    - Supervisor pattern - The Orchestrator's handle function receives all messages, processes them, and then calls agent_ref.send_message(...). Integrate `rig` as the local agent framework to enforce the Supervisor Orchestrator. 

    - Workflow Mode: The Orchestrator holds a petgraph::Graph. When an agent finishes, it sends a TaskComplete message to the Orchestrator, which looks up the next node in the graph and forwards the result.
    - Mesh Mode: The Orchestrator simply sends a "Directory" message to all agents containing the ActorRef of every other agent. Agents then communicate directly, bypassing the Orchestrator. This requires agents be able to attach a client side tool to properly find other agents to hand off.
    - Add a router scheduler which is similar to supervisor but routes messages using a dedicate light weight semantic router?

  - Support client side tools as a protocol extension
  - Support team local storage and tool, team members can use local tools and storage(via storage tool) to share data within the team

