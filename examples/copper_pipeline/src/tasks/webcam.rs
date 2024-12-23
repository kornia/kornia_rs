use cu29::prelude::*;
use kornia::io::stream::{CameraCapture, V4L2CameraConfig};

use crate::tasks::ImageU8Msg;

struct Webcam {
    cam: CameraCapture,
}

impl Freezable for Webcam {}

impl<'cl> CuSrcTask<'cl> for Webcam {
    type Output = output_msg!('cl, ImageU8Msg);

    fn new(_config: Option<&ComponentConfig>) -> Result<Self, CuError>
    where
        Self: Sized,
    {
        let cam = V4L2CameraConfig::default()
            .with_camera_id(0)
            .build()
            .map_err(|e| CuError::new_with_cause("Failed to build camera", e))?;

        Ok(Self { cam })
    }

    fn start(&mut self, _clock: &RobotClock) -> Result<(), CuError> {
        self.cam
            .start()
            .map_err(|e| CuError::new_with_cause("Failed to start camera", e))
    }

    fn stop(&mut self, _clock: &RobotClock) -> Result<(), CuError> {
        self.cam
            .close()
            .map_err(|e| CuError::new_with_cause("Failed to stop camera", e))
    }

    fn process(&mut self, _clock: &RobotClock, new_msg: &Self::Output) -> Result<(), CuError> {
        let Some(img) = self
            .cam
            .grab()
            .map_err(|e| CuError::new_with_cause("Failed to grab image", e))?
        else {
            return Ok(());
        };

        // TODO: send the image to the next task
        // new_msg = ImageU8Msg { image: img };

        Ok(())
    }
}
