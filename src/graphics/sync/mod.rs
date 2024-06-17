use ash::vk;

pub mod fence;
pub mod present_task;
pub mod semaphore;
pub mod submit_task;

pub trait GPUTaskRunner<T>
where
    T: GPUTask,
{
    fn run_task(self) -> TaskResult<T>;
}

pub trait GPUTask {
    type Output;

    fn wait(&self) -> TaskResult<()> {
        match self.wait_result() {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn wait_result(&self) -> TaskResult<Self::Output>;
}

pub type TaskResult<T> = Result<T, GPUResult>;

#[derive(Debug, Clone, Copy)]
pub enum GPUResult {
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
