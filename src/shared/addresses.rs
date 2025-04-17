use crate::bindings::generated::nvm_ppa_addr;

#[derive(Clone, Copy)]
pub struct PhysicalBlockAddress {
    pub channel: u16,
    pub lun: u8,
    pub plane: u8,
    pub block: u16,
}

impl From<nvm_ppa_addr> for PhysicalBlockAddress {
    fn from(value: nvm_ppa_addr) -> Self {
        PhysicalBlockAddress {
            channel: value.get_channel(),
            lun: value.get_lun(),
            plane: value.get_lun(),
            block: value.get_block(),
        }
    }
}

/// ASSUMES THE FOLLOWING STRUCTURE
/// struct nvm_ppa_addr {
///     /* Generic structure for all addresses */
///     union {
///         struct {
///             uint64_t sec   : 3;
///             uint64_t pl    : 2;
///             uint64_t ch    : 12;
///             uint64_t lun   : 6;
///             uint64_t pg    : 12;
///             uint64_t blk   : 15;
///             uint64_t rsv   : 14;
///         } g;
///
///         uint64_t ppa;
///     };
/// };
///
//TODO fix the bindgen
impl nvm_ppa_addr {
    pub fn get_channel(self) -> u16 {
        ((unsafe { self.__bindgen_anon_1.ppa } >> 5) & 0b1111_1111_1111) as u16
    }

    pub fn get_plane(&self) -> u8 {
        ((unsafe { self.__bindgen_anon_1.ppa } >> 3) & 0b0011) as u8
    }

    pub fn get_lun(&self) -> u8 {
        ((unsafe { self.__bindgen_anon_1.ppa } >> 17) & 0b0011_1111) as u8
    }

    pub fn get_block(&self) -> u16 {
        ((unsafe { self.__bindgen_anon_1.ppa } >> 35) & 0b0111_1111_1111_1111) as u16
    }
}
