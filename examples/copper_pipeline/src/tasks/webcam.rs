use cu29::prelude::*;
use kornia::io::stream::V4L2CameraConfig;

use crate::tasks::ImageU8Msg;

struct Webcam {
    cam: V4L2CameraConfig,
}

impl Freezable for Webcam {}

impl<'cl> CuSrcTask<'cl> for Webcam {
    type Output = output_msg!('cl, ImageU8Msg);

    fn new(_config: Option<&ComponentConfig>) -> Result<Self, CuError>
    where
        Self: Sized,
    {
        Ok(Self {
            cam: V4L2CameraConfig::default(),
        })
    }

    fn process(&mut self, _clock: &RobotClock, msg: &Self::Output) -> Result<(), CuError> {
        Ok(())
    }
}
