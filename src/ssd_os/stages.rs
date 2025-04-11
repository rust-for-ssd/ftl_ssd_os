use bindings::{MAGIC_STAGE, ssd_os_ctrl_fn, ssd_os_stage_fn, stage};

use crate::bindings;
impl stage {
    pub const fn new(
        name: &[u8],
        init: ssd_os_ctrl_fn,
        exit: ssd_os_ctrl_fn,
        stage_fn: ssd_os_stage_fn,
    ) -> Self {
        stage {
            magic: *MAGIC_STAGE,
            name: {
                let mut buf = [0u8; 32];
                let mut i = 0;
                while i < name.len() {
                    buf[i] = name[i];
                    i += 1;
                }
                buf
            },
            init_fn: init,
            exit_fn: exit,
            stage_fn,
        }
    }
}
