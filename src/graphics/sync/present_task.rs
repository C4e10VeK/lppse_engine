use crate::graphics::device::Queue;

use super::{GPUResult, GPUTask, PresentInfo, TaskResult, GPUTaskRunner};

pub fn present(queue: Queue, present_info: PresentInfo) -> Presenter {
    Presenter {
        queue,
        present_info,
    }
}

pub struct Presenter {
    queue: Queue,
    present_info: PresentInfo,
}

impl GPUTaskRunner<PresentTask> for Presenter {
    fn run_task(self) -> TaskResult<PresentTask> {
        let present_info = self.present_info.to_vk();

        let result = self
            .queue
            .present(present_info)
            .map_err(|_| GPUResult::PresentError);

        Ok(PresentTask { result })
    }
}

#[derive(Debug)]
pub struct PresentTask {
    result: TaskResult<bool>,
}

impl GPUTask for PresentTask {
    type Output = bool;

    fn wait_result(&self) -> TaskResult<Self::Output> {
        self.result
    }
}
