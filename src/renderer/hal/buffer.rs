use std::marker::PhantomData;
use std::sync::Arc;
use std::mem::{size_of, size_of_val};

use bytemuck::Pod;
use pretty_type_name::pretty_type_name;
use thiserror::Error;

pub type BufferId = usize;

#[derive(Debug, Error)]
#[error("Invalid buffer id {0}")]
pub struct InvalidBufferId(pub BufferId);

#[derive(Debug)]
pub struct Buffer<T> {
    pub inner: Arc<wgpu::Buffer>,
    capacity: usize,
    _phantom_data: PhantomData<T>,
}

impl<T: Pod> Buffer<T> {
    pub fn new(device: &wgpu::Device, capacity: usize, usage: wgpu::BufferUsages) -> Buffer<T> {
        Buffer {
            inner: Arc::new(Buffer::<T>::new_inner(device, capacity * size_of::<T>(), usage)),
            capacity,
            _phantom_data: PhantomData,
        }
    }

    pub fn fill(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) {
        let bytes_to_write = size_of_val(data);
        if bytes_to_write > self.capacity * size_of::<T>() {
            self.inner = Arc::new(Buffer::<T>::new_inner(device, bytes_to_write, self.inner.usage()));
            self.capacity = data.len();
        }

        if !data.is_empty() {
            queue.write_buffer(
                &self.inner, 
                0, 
                bytemuck::cast_slice(data)
            )
        }
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    pub fn inner(&self) -> Arc<wgpu::Buffer> {
        self.inner.clone()
    }

    fn new_inner(device: &wgpu::Device, capacity: usize, usage: wgpu::BufferUsages) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(format!("Buffer ({:?}, {})", usage, pretty_type_name::<T>()).as_str()),
            size: capacity as u64,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}