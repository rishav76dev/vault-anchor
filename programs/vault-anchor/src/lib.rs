use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("5eJvee5EEi3VZiaYJKKb44RtEq99SfUGwbjje2SWvVuU");

#[program]
pub mod vault_anchor {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;

         emit!(InitializeEvent {
        user: ctx.accounts.user.key(),
        vault_state: ctx.accounts.vault_state.key(),
        vault: ctx.accounts.vault.key(), // Include vault
    });


    // Optional logs for debugging
    msg!("Initialize called by: {}", ctx.accounts.user.key());
    msg!("Vault state: {}", ctx.accounts.vault_state.key());
    msg!("Vault PDA: {}", ctx.accounts.vault.key());
        Ok(())
    }

    pub fn deposit(ctx: Context<VaultPayment>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)?;

        emit!(DepositEvent {
            user: ctx.accounts.user.key(),
            amount,
        });

        Ok(())
    }

    pub fn withdraw(ctx: Context<VaultPayment>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)?;

        emit!(WithdrawEvent {
            user: ctx.accounts.user.key(),
            amount,
        });

        Ok(())
    }

    pub fn close_account(ctx: Context<CloseAccount>) -> Result<()> {
        ctx.accounts.close_account()?;

        emit!(CloseEvent {
            user: ctx.accounts.user.key(),
            vault_state: ctx.accounts.vault_state.key(),
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = [b"state", user.key().as_ref()],
        bump,
        space = VaultState::INIT_SPACE,
    )]
    pub vault_state: Account<'info, VaultState>, //Your program’s on-chain struct (Anchor account) storing metadata like bumps.

    #[account(
        seeds = [b"vault", vault_state.key().as_ref()],
        bump,
    )]
    /// CHECK: PDA account used to hold lamports
    // pub vault: AccountInfo<'info>,
    /// A raw PDA account that will hold lamports (native SOL). It’s not an Anchor Account with a struct — it’s just a plain system account.
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.vault_state.vault_bump = bumps.vault;
        self.vault_state.state_bump = bumps.vault_state;
        Ok(())
        //     //    pub struct InitializeBumps {
        //     pub vault_state: u8,  // bump for vault_state PDA
        //     pub vault: u8,        // bump for vault PDA
        // }
    }
}

#[derive(Accounts)]
pub struct VaultPayment<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> VaultPayment<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };
        let seeds = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer(cpi_ctx, amount)
    }
}

#[derive(Accounts)]
pub struct CloseAccount<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    /// CHECK: PDA account used to hold lamports
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
        close = user,
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>,
}

impl<'info> CloseAccount<'info> {
    pub fn close_account(&mut self) -> Result<()> {
        let lamports = self.vault.lamports();

        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };
        let seeds = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer(cpi_ctx, lamports)?;
        Ok(())
    }
}

#[account]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}

impl Space for VaultState {
    const INIT_SPACE: usize = 8 + 1 + 1; // discriminator + two u8s
}

#[event]
pub struct InitializeEvent {
    pub user: Pubkey,
    pub vault_state: Pubkey,
    pub vault: Pubkey,
}

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct CloseEvent {
    pub user: Pubkey,
    pub vault_state: Pubkey,
}
