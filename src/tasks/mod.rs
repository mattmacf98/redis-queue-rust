pub mod add;
pub mod multiply;
pub mod subtract;

use serde::{Serialize, Deserialize};
use crate::tasks::add::AddTask;
use crate::tasks::multiply::MultiplyTask;
use crate::tasks::subtract::SubtractTask;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    ADD(AddTask),
    MULTIPLY(MultiplyTask),
    SUBTRACT(SubtractTask)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMessage {
    pub task: TaskType
}

