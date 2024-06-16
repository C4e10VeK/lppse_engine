use ash::vk;

use crate::graphics::{device::Queue, sync::GPUResult};

use super::{fence::SharedFence, GPUTask, SubmitInfo, TaskRunner};

pub fn submit(queue: Queue, submit_info: SubmitInfo, fence: Option<SharedFence>) -> Submiter {
    Submiter {
        queue,
        submit_info,
        fence,
    }
}

pub struct Submiter {
    queue: Queue,
    submit_info: SubmitInfo,
    fence: Option<SharedFence>,
}

impl TaskRunner<SubmitTask> for Submiter {
    fn run_task(self) -> super::TaskResult<SubmitTask> {
        let submit_info = self.submit_info.to_vk();

        let fence = match self.fence.as_ref() {
            Some(value) => value.handle(),
            None => vk::Fence::null(),
        };

        self.queue
            .submit(std::slice::from_ref(&submit_info), fence)
            .map_err(|_| GPUResult::SubmitError)?;

        Ok(SubmitTask { fence: self.fence })
    }
}

#[derive(Debug)]
pub struct SubmitTask {
    fence: Option<SharedFence>,
}

impl GPUTask for SubmitTask {
    type Output = ();

    fn wait_result(&self) -> super::TaskResult<Self::Output> {
        if let Some(fence) = self.fence.as_ref() {
            fence.wait(u64::MAX).map_err(|_| GPUResult::WaitError)?;
            fence.reset();
        }

        Ok(())
    }
}
