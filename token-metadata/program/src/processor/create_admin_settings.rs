use crate::{state::AdminSettings, CreateAdminSettings};
use anchor_lang::prelude::*;

impl<'info> CreateAdminSettings<'info> {
    pub fn process(&mut self, data: AdminSettings) -> Result<()> {
        msg!("Create admin settings");

        *self.admin_settings = data;
        Ok(())
    }
}
