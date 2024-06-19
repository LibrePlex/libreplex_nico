/*
    Nico is an abstraction layer built on
    top of nifty OSS and core.

    Enables:

    1) serialization
    2) deserialization
    3) common ix interface for: transfers (burn/mint etc to follow)


*/

use anchor_lang::prelude::*;

use anchor_lang::AnchorSerialize;


#[derive(Debug, Clone)]
pub struct Nico {
    pub nico_type: u8,
}

#[cfg(feature = "idl-build")]
impl anchor_lang::IdlBuild for Nico {}

impl anchor_lang::AccountSerialize for Nico {
    fn try_serialize<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<()> {
        if writer
            .write_all(match &self.nico_type {
                NICO_TYPE_CORE => &CORE_DISCRIMINATOR,
                NICO_TYPE_NIFTY_ => &NIFTY_DISCRIMINATOR,
                _ => {
                    panic!("Unexpected NICO type")
                }
            })
            .is_err()
        {
            return Err(anchor_lang::error::ErrorCode::AccountDidNotSerialize.into());
        }

        Ok(())
    }
}
