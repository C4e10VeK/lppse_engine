use ash::vk;

use super::device::Queue;

pub mod fence;
pub mod semaphore;
pub mod submit_task;

pub trait GPUTaskRunner {
    type Output: GPUTask;

    fn run_task(self) -> TaskResult<Self::Output>;
}

pub fn task_from_runner<T>(runner: impl GPUTaskRunner<Output = T>) -> TaskResult<T>
where
    T: GPUTask,
{
    runner.run_task()
}

pub trait GPUTask {
    type Output;

    fn run(&self, queue: Queue) -> TaskResult<()>;

    fn wait(&self) -> TaskResult<()> {
        match self.wait_result() {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn wait_result(&self) -> TaskResult<Self::Output>;

    fn get_signal_semaphore(&self) -> vk::Semaphore;

    fn then_present(&self, queue: Queue, swapchain: vk::SwapchainKHR, image_index: u32) -> TaskResult<bool> {
        let wait_semaphore = self.get_signal_semaphore();

        let info = PresentInfo {
            swapchain,
            image_index,
            wait_semaphore,
        };

        queue.present(info.to_vk())
            .map_err(|_| GPUTaskError::PresentError)
    }
}

pub type TaskResult<T> = Result<T, GPUTaskError>;

#[derive(Debug, Clone, Copy)]
pub enum GPUTaskError {
    SubmitError,
    PresentError,
    AcquireError,
    WaitError,
}

#[derive(Debug)]
pub struct SubmitInfo {
    pub wait_semaphore: vk::Semaphore,
    pub signal_semaphore: vk::Semaphore,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub wait_dst_stage_mask: vk::PipelineStageFlags,
}

impl SubmitInfo {
    pub fn to_vk(&self) -> vk::SubmitInfo<'_> {
        vk::SubmitInfo::default()
            .command_buffers(&self.command_buffers)
            .wait_dst_stage_mask(std::slice::from_ref(&self.wait_dst_stage_mask))
            .signal_semaphores(std::slice::from_ref(&self.signal_semaphore))
            .wait_semaphores(std::slice::from_ref(&self.wait_semaphore))
    }
}

#[derive(Debug)]
pub struct PresentInfo {
    pub swapchain: vk::SwapchainKHR,
    pub wait_semaphore: vk::Semaphore,
    pub image_index: u32,
}

impl PresentInfo {
    pub fn to_vk(&self) -> vk::PresentInfoKHR<'_> {
        vk::PresentInfoKHR::default()
            .wait_semaphores(std::slice::from_ref(&self.wait_semaphore))
            .swapchains(std::slice::from_ref(&self.swapchain))
            .image_indices(std::slice::from_ref(&self.image_index))
    }
}
