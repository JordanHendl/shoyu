use dashi::utils::*;
use dashi::*;

pub struct HotBuffer {
    front: Handle<Buffer>,
    staging: Handle<Buffer>,
    ptr: *mut u8,
    size: usize,
}

impl HotBuffer {
    pub fn new(ctx: &mut Context, info: &BufferInfo) -> Self {
        let mut cln = info.clone();
        let front = ctx.make_buffer(&cln).unwrap();
        cln.visibility = MemoryVisibility::CpuAndGpu;
        let staging = ctx.make_buffer(&cln).unwrap();

        let mapped = ctx.map_buffer_mut::<u8>(staging).unwrap();

        Self {
            front,
            staging,
            ptr: mapped.as_mut_ptr(),
            size: mapped.len(),
        }
    }

    pub fn raw(&self) -> Handle<Buffer> {
        self.front
    }

    pub fn sync(&mut self, list: &mut CommandList) {
        list.copy_buffers(&BufferCopy {
            src: self.staging,
            dst: self.front,
            src_offset: 0,
            dst_offset: 0,
            size: self.size,
        });
    }

    pub fn slice<T>(&self) -> &[T] {
        let typed_map: *const T = unsafe { std::mem::transmute(self.ptr) };
        unsafe { std::slice::from_raw_parts(typed_map, self.size / std::mem::size_of::<T>()) }
    }

    pub fn slice_mut<T>(&self) -> &mut [T] {
        let typed_map: *mut T = unsafe { std::mem::transmute(self.ptr) };
        unsafe { std::slice::from_raw_parts_mut(typed_map, self.size / std::mem::size_of::<T>()) }
    }
}
