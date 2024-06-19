use ash::vk;

use crate::graphics::{device::Queue, sync::GPUTaskError};

use super::{fence::SharedFence, GPUTask, GPUTaskRunner, SubmitInfo};

pub fn submit(
    queue: Queue,
    submit_info: SubmitInfo,
    fence: Option<SharedFence>,
) -> SubmitTaskRunner {
    SubmitTaskRunner {
        queue,
        submit_info,
        fence,
    }
}

pub struct SubmitTaskRunner {
    queue: Queue,
    submit_info: SubmitInfo,
    fence: Option<SharedFence>,
}

impl GPUTaskRunner for SubmitTaskRunner {
    type Output = SubmitTask;

    fn run_task(self) -> super::TaskResult<Self::Output> {
        let task = SubmitTask { 
            info: self.submit_info,
            fence: self.fence,
        };

        task.run(self.queue)?;

        Ok(task)
    }
}

#[derive(Debug)]
pub struct SubmitTask {
    fence: Option<SharedFence>,
    info: SubmitInfo,
}

impl SubmitTask {
    fn get_raw_fence(&self) -> vk::Fence {
        match self.fence.clone() {
            Some(value) => value.handle(),
            None => vk::Fence::null(),
        }
    } 
}

impl GPUTask for SubmitTask {
    type Output = ();

    fn run(&self, queue: Queue) -> super::TaskResult<()> {
        let info = self.info.to_vk();

        let fence = self.get_raw_fence();

        queue.submit(std::slice::from_ref(&info), fence)
            .map_err(|_| GPUTaskError::SubmitError)
    }

    fn wait_result(&self) -> super::TaskResult<Self::Output> {
        if let Some(fence) = self.fence.as_ref() {
            fence.wait(u64::MAX).map_err(|_| GPUTaskError::WaitError)?;
            fence.reset();
        }

        Ok(())
    }

    fn get_signal_semaphore(&self) -> vk::Semaphore {
        self.info.signal_semaphore
    }
}
