use crate::bindings::generated::nvm_ppa_addr;

#[derive(Clone, Copy)]
pub struct PhysicalBlockAddress {
    pub channel: u64,
    pub lun: u64,
    pub plane: u64,
    pub block: u64,
}

impl From<nvm_ppa_addr> for PhysicalBlockAddress {
    fn from(value: nvm_ppa_addr) -> Self {
        unsafe {
            PhysicalBlockAddress {
                channel: value.__bindgen_anon_1.g.ch(),
                lun: value.__bindgen_anon_1.g.lun(),
                plane: value.__bindgen_anon_1.g.pl(),
                block: value.__bindgen_anon_1.g.blk(),
            }
        }
    }
}
