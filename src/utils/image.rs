pub struct SizedImage {
    pub handle: dashi::utils::Handle<dashi::Image>,
    pub view: dashi::utils::Handle<dashi::ImageView>,
    pub dim: [u32; 3],
    pub format: dashi::Format,
}
