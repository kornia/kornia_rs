use cu29::prelude::*;

use crate::tasks::ImageU8Msg;

struct RerunViz {
    rec: rerun::RecordingStream,
}

impl Freezable for RerunViz {}

impl<'cl> CuSinkTask<'cl> for RerunViz {
    type Input = input_msg!('cl, ImageU8Msg);

    fn new(_config: Option<&ComponentConfig>) -> Result<Self, CuError>
    where
        Self: Sized,
    {
        Ok(Self {
            rec: rerun::RecordingStreamBuilder::new("kornia_app")
                .spawn()
                .map_err(|e| CuError::new_with_cause("Failed to spawn rerun stream", e))?,
        })
    }

    fn process(&mut self, _clock: &RobotClock, msg: &Self::Input) -> Result<(), CuError> {
        Ok(())
    }
}
