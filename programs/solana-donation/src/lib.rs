use anchor_lang::prelude::*;

declare_id!("EDuB9MCqPUAbjfTsrRak2v1V99G45z4nBV6vCcLjLx6v");

#[program]
pub mod solana_donation {
    use anchor_lang::solana_program::{program::invoke, system_instruction::transfer};

    use super::*;
    use std::u64;

    pub fn initialize(ctx: Context<Initialize>, target: u64) -> Result<()> {
        require!(target > 0, DonateErrors::ZeroLamports);
        let donate_platform = &mut ctx.accounts.donate_platform;
        donate_platform.authority = ctx.accounts.authority.key();
        donate_platform.target = target;
        donate_platform.collected = 0;
        donate_platform.id_counter = 0;
        Ok(())
    }

    pub fn send(ctx: Context<Send>, id: u64, amount: u64) -> Result<()> {
        require!(amount > 0, DonateErrors::ZeroLamports);
        let donate_platform = &ctx.accounts.donate_platform;
        require!(
            id <= donate_platform.id_counter,
            DonateErrors::IDBiggerThanCounter
        );

        let donator = &ctx.accounts.donator;

        let collected = donate_platform.collected;
        let target = donate_platform.target;
        require!(target > collected, DonateErrors::TargetReached);

        let (from, from_info) = (&donator.key(), donator.to_account_info());
        let (to, to_info) = (&donate_platform.key(), donate_platform.to_account_info());

        invoke(&transfer(from, to, amount), &[from_info, to_info])?;

        let donate_platform = &mut ctx.accounts.donate_platform;
        let donator_acc = &mut ctx.accounts.donator_acc;

        let mut _id = id;
        if _id == 0 {
            _id = donate_platform.id_counter;
        }

        if _id == donate_platform.id_counter {
            donator_acc.address = ctx.accounts.donator.key();
            donator_acc.amount = 0;

            donate_platform.id_counter += 1;
        }

        donator_acc.amount += amount;
        donate_platform.collected += amount;

        emit!(DonationEvent {
            at: Clock::get()?.unix_timestamp,
            amount: amount,
            platform_after: donate_platform.collected,
            from: donator_acc.address
        });
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let collected = ctx.accounts.donate_platform.collected;
        require!(collected > 0, DonateErrors::NoCollectedLamports);

        let from = ctx.accounts.donate_platform.to_account_info();
        let to = ctx.accounts.authority.to_account_info();

        let rent_exemption = Rent::get()?.minimum_balance(Donates::SIZE);
        let withdraw_amount = **from.lamports.borrow() - rent_exemption;
        // require!(withdraw_amount < collected, DonateErrors::NoLamportsForRent);

        **from.try_borrow_mut_lamports()? -= withdraw_amount;
        **to.try_borrow_mut_lamports()? += withdraw_amount;
        ctx.accounts.donate_platform.collected = 0;

        emit!(WithdrawEvent {
            at: Clock::get()?.unix_timestamp,
            amount: withdraw_amount
        });

        Ok(())
    }
}

//-----------------------------------------------------------------------------------------------------------------------------------------------
#[derive(Accounts)]
#[instruction(target: u64)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = Donates::SIZE,
        seeds = [b"donate_platform", authority.key().as_ref()],
        bump
    )]
    pub donate_platform: Account<'info, Donates>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(id: u64, amount: u64)]
pub struct Send<'info> {
    #[account(
        init_if_needed,
        payer = donator,
        space = Donator::SIZE,
        seeds = [
            b"donate_platform_donator",
            donate_platform.key().as_ref(),
            id.to_string().as_bytes()
        ],
        bump
    )]
    pub donator_acc: Account<'info, Donator>,

    #[account(
        mut,
        seeds = [b"donate_platform", donate_platform.authority.key().as_ref()],
        bump
    )]
    pub donate_platform: Account<'info, Donates>,
    #[account(mut)]
    pub donator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"donate_platform", donate_platform.authority.key().as_ref()],
        bump
    )]
    pub donate_platform: Account<'info, Donates>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// -------------------------------------------------------------------------------------------------------------------------------------------------------------------
#[account]
pub struct Donates {
    pub authority: Pubkey,
    pub target: u64,
    pub collected: u64,
    pub id_counter: u64,
}
impl Donates {
    pub const SIZE: usize = 8 + 32 + 8 * 3;
}

#[account]
pub struct Donator {
    pub address: Pubkey,
    pub amount: u64,
}

impl Donator {
    pub const SIZE: usize = 8 + 32 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct DonatorStruct {
    pub address: Pubkey,
    pub amount: u64,
}

//--------------------------------------------------------------------------------------------------------------------------------------------------------------------

#[error_code]
pub enum DonateErrors {
    #[msg("Amount of lamports must be more than zero")]
    ZeroLamports,
    #[msg("Impossible to withdraw. No lamports were collected")]
    NoCollectedLamports,
    #[msg("The target was reached")]
    TargetReached,
    #[msg("Impossible to withdraw. Not enough lamports to pay rent")]
    NoLamportsForRent,
    #[msg("Passed ID is bigger than current ID counter")]
    IDBiggerThanCounter,
}

#[event]
pub struct DonationEvent {
    at: i64,
    amount: u64,
    platform_after: u64,
    from: Pubkey,
}

#[event]
pub struct WithdrawEvent {
    at: i64,
    amount: u64,
}
