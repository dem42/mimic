use crate::buffers::memory::MemoryCopyable;
use ash::vk;
//////////////////////// Consts ///////////////////////
pub const INDEX_TYPE_VK_TYPE: vk::IndexType = vk::IndexType::UINT32;
//////////////////////// Types ///////////////////////
pub type IndexType = u32;
//////////////////////// Impls ///////////////////////
impl MemoryCopyable for [IndexType] {
    unsafe fn copy_to_mapped_memory(&self, data_target_ptr: *mut std::ffi::c_void) {
        let data_ptr = data_target_ptr as *mut IndexType;
        data_ptr.copy_from_nonoverlapping(self.as_ptr(), self.len());
    }
}
