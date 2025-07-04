use anyhow::Result;
use pollster::block_on;

#[derive(Clone, Debug)]
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuContext {
    pub fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(&Default::default());
        let adapter = block_on(instance.request_adapter(&Default::default()))?;
        let (device, queue) = block_on(adapter.request_device(&Default::default()))?;
        Ok(Self {
            instance,
            adapter,
            device,
            queue
        })
    }
}