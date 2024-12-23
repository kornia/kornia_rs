mod rerun_viz;
pub use rerun_viz::*;

mod webcam;
pub use webcam::*;

type ImageU8 = kornia::image::Image<u8, 3>;

#[derive(Clone)]
pub struct ImageU8Msg {
    pub image: ImageU8,
}

impl std::fmt::Debug for ImageU8Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImageU8Msg(size: {:?})", self.image.size())
    }
}

impl Default for ImageU8Msg {
    fn default() -> Self {
        Self {
            image: ImageU8::new([1, 1].into(), vec![0; 3]).unwrap(),
        }
    }
}
