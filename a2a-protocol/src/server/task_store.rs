//! Task storage and management

use crate::{A2aError, A2aResult, Task, TaskState, TaskStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory task storage
#[derive(Debug, Clone)]
pub struct TaskStore {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

impl TaskStore {
    /// Create a new task store
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a task
    pub async fn store(&self, task: Task) -> A2aResult<()> {
        let task_id = task.id.clone();
        self.tasks.write().await.insert(task_id, task);
        Ok(())
    }

    /// Retrieve a task by ID
    pub async fn get(&self, task_id: &str) -> A2aResult<Task> {
        self.tasks
            .read()
            .await
            .get(task_id)
            .cloned()
            .ok_or_else(|| A2aError::TaskNotFound {
                task_id: task_id.to_string(),
            })
    }

    /// Update a task
    pub async fn update(&self, task: Task) -> A2aResult<()> {
        let task_id = task.id.clone();
        let mut tasks = self.tasks.write().await;

        if tasks.contains_key(&task_id) {
            tasks.insert(task_id, task);
            Ok(())
        } else {
            Err(A2aError::TaskNotFound { task_id })
        }
    }

    /// Get task status
    pub async fn get_status(&self, task_id: &str) -> A2aResult<TaskStatus> {
        let task = self.get(task_id).await?;
        Ok(task.status)
    }

    /// Update task state
    pub async fn update_state(&self, task_id: &str, new_state: TaskState) -> A2aResult<TaskStatus> {
        let mut tasks = self.tasks.write().await;

        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| A2aError::TaskNotFound {
                task_id: task_id.to_string(),
            })?;

        // Validate state transition
        self.validate_state_transition(&task.status.state, &new_state)?;

        // Create new status
        let new_status = TaskStatus::new(new_state.clone());

        // Add current status to history
        if task.history.is_none() {
            task.history = Some(vec![task.status.clone()]);
        } else {
            task.history.as_mut().unwrap().push(task.status.clone());
        }

        // Update to new status
        task.status = new_status.clone();

        Ok(new_status)
    }

    /// Cancel a task
    pub async fn cancel(&self, task_id: &str, reason: Option<String>) -> A2aResult<TaskStatus> {
        let mut tasks = self.tasks.write().await;

        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| A2aError::TaskNotFound {
                task_id: task_id.to_string(),
            })?;

        // Check if task can be cancelled
        match task.status.state {
            TaskState::Completed | TaskState::Cancelled | TaskState::Failed => {
                return Err(A2aError::TaskNotCancelable {
                    task_id: task_id.to_string(),
                    state: task.status.state.clone(),
                });
            }
            _ => {}
        }

        // Create new cancelled status
        let new_status = if let Some(reason_text) = reason {
            TaskStatus::new(TaskState::Cancelled).with_reason(reason_text)
        } else {
            TaskStatus::new(TaskState::Cancelled)
        };

        // Add current status to history
        if task.history.is_none() {
            task.history = Some(vec![task.status.clone()]);
        } else {
            task.history.as_mut().unwrap().push(task.status.clone());
        }

        // Update to cancelled status
        task.status = new_status.clone();

        Ok(new_status)
    }

    /// List all tasks
    pub async fn list_all(&self) -> A2aResult<Vec<Task>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().cloned().collect())
    }

    /// List tasks by state
    pub async fn list_by_state(&self, state: TaskState) -> A2aResult<Vec<Task>> {
        let tasks = self.tasks.read().await;
        Ok(tasks
            .values()
            .filter(|t| t.status.state == state)
            .cloned()
            .collect())
    }

    /// Delete a task
    pub async fn delete(&self, task_id: &str) -> A2aResult<Task> {
        self.tasks
            .write()
            .await
            .remove(task_id)
            .ok_or_else(|| A2aError::TaskNotFound {
                task_id: task_id.to_string(),
            })
    }

    /// Get task count
    pub async fn count(&self) -> usize {
        self.tasks.read().await.len()
    }

    /// Clear all tasks (for testing)
    pub async fn clear(&self) {
        self.tasks.write().await.clear();
    }

    /// Validate state transition
    fn validate_state_transition(&self, from: &TaskState, to: &TaskState) -> A2aResult<()> {
        // Define valid transitions
        let valid_transitions: HashMap<TaskState, Vec<TaskState>> = [
            (
                TaskState::Pending,
                vec![TaskState::Working, TaskState::Cancelled, TaskState::Failed],
            ),
            (
                TaskState::Working,
                vec![
                    TaskState::Blocked,
                    TaskState::Review,
                    TaskState::Completed,
                    TaskState::Failed,
                    TaskState::Cancelled,
                    TaskState::Suspended,
                ],
            ),
            (
                TaskState::Blocked,
                vec![TaskState::Working, TaskState::Failed, TaskState::Cancelled],
            ),
            (
                TaskState::Review,
                vec![
                    TaskState::Working,
                    TaskState::Completed,
                    TaskState::Failed,
                    TaskState::Cancelled,
                ],
            ),
            (
                TaskState::Suspended,
                vec![TaskState::Working, TaskState::Cancelled, TaskState::Failed],
            ),
            // Terminal states (no transitions out)
            (TaskState::Completed, vec![]),
            (TaskState::Cancelled, vec![]),
            (TaskState::Failed, vec![]),
        ]
        .into_iter()
        .collect();

        if let Some(allowed_states) = valid_transitions.get(from) {
            if allowed_states.contains(to) || from == to {
                Ok(())
            } else {
                Err(A2aError::UnsupportedOperation(format!(
                    "Invalid state transition from {:?} to {:?}",
                    from, to
                )))
            }
        } else {
            Ok(()) // Allow if not defined
        }
    }
}

impl Default for TaskStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve_task() {
        let store = TaskStore::new();
        let task = Task::new("Test task");

        let task_id = task.id.clone();
        store.store(task.clone()).await.unwrap();

        let retrieved = store.get(&task_id).await.unwrap();
        assert_eq!(retrieved.id, task_id);
        assert_eq!(retrieved.status.state, TaskState::Pending);
    }

    #[tokio::test]
    async fn test_get_nonexistent_task() {
        let store = TaskStore::new();
        let result = store.get("nonexistent").await;
        match result {
            Err(A2aError::TaskNotFound { task_id }) => assert_eq!(task_id, "nonexistent"),
            other => panic!("Expected TaskNotFound error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_update_task_state() {
        let store = TaskStore::new();
        let task = Task::new("Test task");
        let task_id = task.id.clone();

        store.store(task).await.unwrap();

        // Update state: Pending -> Working
        let status = store
            .update_state(&task_id, TaskState::Working)
            .await
            .unwrap();
        assert_eq!(status.state, TaskState::Working);
        assert!(store.get(&task_id).await.unwrap().history.is_some());

        // Verify task was updated
        let task = store.get(&task_id).await.unwrap();
        assert_eq!(task.status.state, TaskState::Working);
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let store = TaskStore::new();
        let task = Task::new("Test task");
        let task_id = task.id.clone();

        store.store(task).await.unwrap();
        store
            .update_state(&task_id, TaskState::Working)
            .await
            .unwrap();

        // Cancel task
        let status = store
            .cancel(&task_id, Some("User requested".to_string()))
            .await
            .unwrap();
        assert_eq!(status.state, TaskState::Cancelled);
        assert!(status.reason.is_some());
        assert_eq!(status.reason.as_ref().unwrap(), "User requested");
    }

    #[tokio::test]
    async fn test_cannot_cancel_completed_task() {
        let store = TaskStore::new();
        let task = Task::new("Test task");
        let task_id = task.id.clone();

        store.store(task).await.unwrap();
        store
            .update_state(&task_id, TaskState::Working)
            .await
            .unwrap();
        store
            .update_state(&task_id, TaskState::Completed)
            .await
            .unwrap();

        // Try to cancel completed task
        let result = store.cancel(&task_id, None).await;
        match result {
            Err(A2aError::TaskNotCancelable { task_id: id, state }) => {
                assert_eq!(id, task_id);
                assert_eq!(state, TaskState::Completed);
            }
            other => panic!("Expected TaskNotCancelable error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_invalid_state_transition() {
        let store = TaskStore::new();
        let task = Task::new("Test task");
        let task_id = task.id.clone();

        store.store(task).await.unwrap();

        // Try invalid transition: Pending -> Completed (should go through Working first)
        let result = store.update_state(&task_id, TaskState::Completed).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_tasks_by_state() {
        let store = TaskStore::new();

        let task1 = Task::new("Task 1");
        let task2 = Task::new("Task 2");
        let task3 = Task::new("Task 3");

        store.store(task1.clone()).await.unwrap();
        store.store(task2.clone()).await.unwrap();
        store.store(task3.clone()).await.unwrap();

        store
            .update_state(&task2.id, TaskState::Working)
            .await
            .unwrap();
        store
            .update_state(&task3.id, TaskState::Working)
            .await
            .unwrap();

        let pending_tasks = store.list_by_state(TaskState::Pending).await.unwrap();
        assert_eq!(pending_tasks.len(), 1);

        let working_tasks = store.list_by_state(TaskState::Working).await.unwrap();
        assert_eq!(working_tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_task_count() {
        let store = TaskStore::new();
        assert_eq!(store.count().await, 0);

        store.store(Task::new("Task 1")).await.unwrap();
        assert_eq!(store.count().await, 1);

        store.store(Task::new("Task 2")).await.unwrap();
        assert_eq!(store.count().await, 2);
    }

    #[tokio::test]
    async fn test_delete_task() {
        let store = TaskStore::new();
        let task = Task::new("Test task");
        let task_id = task.id.clone();

        store.store(task).await.unwrap();
        assert_eq!(store.count().await, 1);

        let deleted = store.delete(&task_id).await.unwrap();
        assert_eq!(deleted.id, task_id);
        assert_eq!(store.count().await, 0);
    }
}
