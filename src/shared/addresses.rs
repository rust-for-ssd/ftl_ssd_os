use core::mem::MaybeUninit;

use crate::bindings::generated::nvm_ppa_addr;

// TODO: why u64?
#[derive(Clone, Copy, PartialEq, Debug)]
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

// TODO: why u64?
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PhysicalPageAddress {
    pub channel: u64,
    pub lun: u64,
    pub plane: u64,
    pub block: u64,
    pub page: u64,
}

impl From<nvm_ppa_addr> for PhysicalPageAddress {
    fn from(value: nvm_ppa_addr) -> Self {
        unsafe {
            PhysicalPageAddress {
                channel: value.__bindgen_anon_1.g.ch(),
                lun: value.__bindgen_anon_1.g.lun(),
                plane: value.__bindgen_anon_1.g.pl(),
                block: value.__bindgen_anon_1.g.blk(),
                page: value.__bindgen_anon_1.g.pg(),
            }
        }
    }
}

impl From<PhysicalPageAddress> for nvm_ppa_addr {
    fn from(addr: PhysicalPageAddress) -> Self {
        // SAFETY: We zero the struct so that all padding and unused bits start at 0.
        // Then we explicitly set each field via the generated setter methods.
        let mut ppa: MaybeUninit<nvm_ppa_addr> = MaybeUninit::zeroed();
        let mut ppa = unsafe { ppa.assume_init() };
        unsafe {
            // these `set_*` methods come from bindgen for each bit-field
            ppa.__bindgen_anon_1.g.set_ch(addr.channel);
            ppa.__bindgen_anon_1.g.set_lun(addr.lun);
            ppa.__bindgen_anon_1.g.set_pl(addr.plane);
            ppa.__bindgen_anon_1.g.set_blk(addr.block);
            ppa.__bindgen_anon_1.g.set_pg(addr.page);
        }
        ppa
    }
}
