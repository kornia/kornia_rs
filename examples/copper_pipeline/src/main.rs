use cu29::prelude::*;
use cu29_helpers::basic_copper_setup;

const SLAB_SIZE: Option<usize> = Some(100 * 1024 * 1024);

#[copper_runtime(config = "kornia_app.ron")]
struct KorniaApplication {}

fn main() {
    //const PACKET_SIZE: usize = size_of::<Packet>();
    let tmp_dir = tempfile::TempDir::new().expect("could not create a tmp dir");
    let logger_path = tmp_dir.path().join("kornia_app.copper");
    let copper_ctx =
        basic_copper_setup(&logger_path, SLAB_SIZE, false, None).expect("Failed to setup copper.");

    let mut application =
        KorniaApplication::new(copper_ctx.clock.clone(), copper_ctx.unified_logger.clone())
            .expect("Failed to create application.");
    application
        .start_all_tasks()
        .expect("Failed to start all tasks.");

    application.run();
}
