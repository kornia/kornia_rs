use cu29::prelude::*;
use kornia::imgproc;

use super::cu_image::ImageRGBU8Msg;

pub struct Sobel;

impl Freezable for Sobel {}

impl<'cl> CuTask<'cl> for Sobel {
    type Input = input_msg!('cl, ImageRGBU8Msg);
    type Output = output_msg!('cl, ImageRGBU8Msg);

    fn new(config: Option<&ComponentConfig>) -> Result<Self, CuError>
    where
        Self: Sized,
    {
        Ok(Self {})
    }

    fn start(&mut self, _clock: &RobotClock) -> Result<(), CuError> {
        Ok(())
    }

    fn stop(&mut self, _clock: &RobotClock) -> Result<(), CuError> {
        Ok(())
    }

    fn process(
        &mut self,
        _clock: &RobotClock,
        input: Self::Input,
        output: Self::Output,
    ) -> Result<(), CuError> {
        let Some(src) = input.payload() else {
            return Ok(());
        };

        let dst = imgproc::filters::sobel(src, kornia::core::types::BorderType::REPLICATE);

        Ok(())
    }
}
