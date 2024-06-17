use crate::graphics::device::Queue;

use super::{GPUTask, GPUTaskError, GPUTaskRunner, PresentInfo, TaskResult};

pub fn present(queue: Queue, present_info: PresentInfo) -> PresentTaskRunner {
    PresentTaskRunner {
        queue,
        present_info,
    }
}

pub struct PresentTaskRunner {
    queue: Queue,
    present_info: PresentInfo,
}

impl GPUTaskRunner<PresentTask> for PresentTaskRunner {
    fn run_task(self) -> TaskResult<PresentTask> {
        let present_info = self.present_info.to_vk();

        let result = self
            .queue
            .present(present_info)
            .map_err(|_| GPUTaskError::PresentError);

        Ok(PresentTask { result })
    }
}

#[derive(Debug)]
pub struct PresentTask {
    result: TaskResult<bool>,
}

impl GPUTask for PresentTask {
    type Output = bool;

    #[inline]
    fn wait_result(&self) -> TaskResult<Self::Output> {
        self.result
    }
}
