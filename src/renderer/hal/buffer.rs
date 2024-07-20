use std::marker::PhantomData;
use std::sync::Arc;
use std::mem::{size_of, size_of_val};

use bytemuck::Pod;
use pretty_type_name::pretty_type_name;
use thiserror::Error;

use crate::renderer::error::RenderError;

pub type BufferId = usize;

#[derive(Debug, Error)]
#[error("Invalid buffer id {0}")]
pub struct InvalidBufferId(pub BufferId);

/// A generic buffer used for storing data on the GPU.
/// 
/// # Type Parameters
/// 
/// * `T` - The type of data stored in the buffer. Must implement the `Pod` trait.
#[derive(Debug)]
pub struct Buffer<T> {
    pub inner: Arc<wgpu::Buffer>,
    capacity: usize,
    _phantom_data: PhantomData<T>,
}

impl<T: Pod> Buffer<T> {
    /// Creates a new buffer with the given capacity and usage.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the wgpu device.
    /// * `capacity` - The capacity of the buffer in number of elements of type `T`.
    /// * `usage` - The buffer usage flags.
    ///
    /// # Returns
    ///
    /// A new instance of `Buffer<T>`.
    pub fn new(device: &wgpu::Device, capacity: usize, usage: wgpu::BufferUsages) -> Buffer<T> {
        Buffer {
            inner: Arc::new(Buffer::<T>::new_inner(device, capacity * size_of::<T>(), usage)),
            capacity,
            _phantom_data: PhantomData,
        }
    }

    /// Fills the buffer with the given data, ensuring the data fits within the buffer capacity.
    ///
    /// # Arguments
    ///
    /// * `queue` - A reference to the wgpu queue.
    /// * `data` - A slice of data to be written to the buffer.
    ///
    /// # Errors
    ///
    /// Returns `RenderError::BufferOverflow` if the data length exceeds the buffer capacity.
    pub fn fill_exact(&self, queue: &wgpu::Queue, data: &[T]) -> Result<(), RenderError> {
        if data.len() > self.capacity {
            return Err(RenderError::BufferOverflow(data.len()));
        }

        if !data.is_empty() {
            queue.write_buffer(&self.inner, 0, bytemuck::cast_slice(data));
        }

        Ok(())
    }

    /// Fills the buffer with the given data, resizing the buffer if necessary.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the wgpu device.
    /// * `queue` - A reference to the wgpu queue.
    /// * `data` - A slice of data to be written to the buffer.
    pub fn fill(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) {
        let bytes_to_write = size_of_val(data);
        if bytes_to_write > self.capacity * size_of::<T>() {
            self.inner = Arc::new(Buffer::<T>::new_inner(device, bytes_to_write, self.inner.usage()));
            self.capacity = data.len();
        }

        self.fill_exact(queue, data).unwrap();
    }
    
    /// Returns the capacity of the buffer.
    ///
    /// # Returns
    ///
    /// The capacity of the buffer in number of elements of type `T`.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Returns a clone of the inner wgpu buffer.
    ///
    /// # Returns
    ///
    /// An `Arc` pointing to the inner wgpu buffer.
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
