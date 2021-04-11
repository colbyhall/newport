use super::Device;

use ash::vk;
use ash::version::DeviceV1_0;

use std::sync::Arc;

#[derive(Clone)]
pub struct Receipt {
    owner:   Arc<Device>,
    id:      usize,
}

impl Receipt {
    pub(super) fn new(owner: Arc<Device>, id: usize) -> Self {
        Self{
            owner:   owner,
            id:      id
        }
    }

    pub(crate) fn get(&self) -> Option<(vk::Semaphore, vk::Fence)> {
        let work = self.owner.work.lock().unwrap();
        let result = work.in_queue.get(&self.id)?;
        Some((result.semaphore, result.fence))
    }
    
    pub fn wait(self) -> bool {
        {
            let work = self.owner.work.lock().unwrap();
    
            let entry = work.in_queue.get(&self.id);
            if entry.is_none() { return false; }
            let entry = entry.unwrap();
    
            unsafe {
                self.owner.logical.wait_for_fences(&[entry.fence], true, u64::MAX).unwrap()
            };
        }
    
        self.owner.remove_finished_work();
    
        return true;
    }
    
    pub fn is_finished(&self) -> bool {
        let work = self.owner.work.lock().unwrap();
        let result = work.in_queue.get(&self.id);
        result.is_none()
    }
}