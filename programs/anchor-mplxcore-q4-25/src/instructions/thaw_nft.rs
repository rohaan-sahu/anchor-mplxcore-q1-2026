use anchor_lang::prelude::*;
use mpl_core::{
    instructions::UpdatePluginV1CpiBuilder,
    types::{FreezeDelegate, Plugin},
    ID as CORE_PROGRAM_ID,
};

use crate::{error::MPLXCoreError, state::CollectionAuthority};

#[derive(Accounts)]
pub struct ThawNft<'info> {
   // CHECK: This will also be checked by core
    #[account(address = CORE_PROGRAM_ID)]
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,

   #[account(mut)]
    pub minter: Signer<'info>,

    #[account(mut, constraint = asset.data_is_empty() @ MPLXCoreError::AssetAlreadyInitialized)]
    pub asset: Signer<'info>,

    #[account(
        mut,
        constraint = collection.owner == &CORE_PROGRAM_ID @ MPLXCoreError::InvalidCollection,
        constraint = !collection.data_is_empty() @ MPLXCoreError::CollectionNotInitialized
    )]
    /// CHECK: This will also be checked by core
    pub collection: UncheckedAccount<'info>,

    #[account(
        seeds = [b"collection_authority", collection.key().as_ref()],
        bump = collection_authority.bump,
    )]
    pub collection_authority: Account<'info, CollectionAuthority>,
}

impl<'info> ThawNft<'info> {
    pub fn thaw_nft(&mut self) -> Result<()> {
        let (collection_authority_pda, _collection_authority_pda_bump) = Pubkey::find_program_address(
            &[b"collection_authority",self.collection.key.as_ref()],
            &crate::ID
        );

        require_keys_eq!(
            collection_authority_pda,
            self.collection_authority.key(),
            MPLXCoreError::InvalidCollection
        );
        require_keys_eq!(
            self.collection_authority.creator,
            self.minter.key(),
            MPLXCoreError::NotAuthorized
        );

        let binding = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"collection_authority".as_ref(),
            binding.as_ref(),
            &[self.collection_authority.bump],
        ]];

        UpdatePluginV1CpiBuilder::new(&self.core_program)
            .asset(&self.asset)
            .collection(Some(&self.collection))
            .payer(&self.minter.to_account_info())
            .authority(Some(&self.collection_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate {frozen: false}))
            .invoke_signed(signer_seeds)?;


        Ok(())
    }
}
