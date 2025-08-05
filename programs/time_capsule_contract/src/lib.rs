use solana_security_txt::security_txt;
use anchor_lang::prelude::*;

security_txt! {
    name: "Time Capsule Contract",
    project_url: "https://github.com/patch75/time-capsule-contract",
    contacts: "email:contact@nobylis.net",
    policy: "https://github.com/patch75/time-capsule-contract/blob/main/security.txt"
}

declare_id!("FXmb9NmMdbtTX4RRKsBzGzSLD6NzxMKoZSgmTKNJ1jhk");

#[program]
pub mod time_capsule_contract {
    use super::*;

    /// Initialiser la configuration du programme
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        price: u64,
        treasury: Pubkey,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.price = price;
        config.authority = ctx.accounts.authority.key();
        config.treasury = treasury;
        Ok(())
    }

    /// Créer une nouvelle capsule temporelle
    pub fn create_time_capsule(
        ctx: Context<CreateTimeCapsule>,
        encrypted_message: String,
        unlock_timestamp: i64,
        recipient_email_hash: String,
        password_hash: String,
        password_hint: String,
        message_title: String,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let config = &ctx.accounts.config;

        // Vérifications de base
        require!(
            unlock_timestamp > clock.unix_timestamp,
            TimeCapsuleError::InvalidUnlockTime
        );
        
        require!(
            encrypted_message.len() <= 5000,
            TimeCapsuleError::MessageTooLong
        );
        
        require!(
            password_hint.len() <= 500,
            TimeCapsuleError::HintTooLong
        );
        
        require!(
            message_title.len() <= 200,
            TimeCapsuleError::TitleTooLong
        );

        // ✅ NOUVELLES VALIDATIONS STRICTES DES HASHES
        require!(
            recipient_email_hash.len() == 64,
            TimeCapsuleError::InvalidEmailHash
        );
        
        require!(
            password_hash.len() == 64,
            TimeCapsuleError::InvalidPasswordHash
        );

        // Vérifier treasury
        require!(
            ctx.accounts.treasury.key() == config.treasury,
            TimeCapsuleError::InvalidTreasury
        );

        // Effectuer le paiement
        let transfer_instruction = anchor_lang::system_program::Transfer {
            from: ctx.accounts.sender.to_account_info(),
            to: ctx.accounts.treasury.to_account_info(),
        };
        
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                transfer_instruction,
            ),
            config.price,
        )?;

        // Initialiser la capsule
        let time_capsule = &mut ctx.accounts.time_capsule;
        time_capsule.sender = ctx.accounts.sender.key();
        time_capsule.encrypted_message = encrypted_message;
        time_capsule.unlock_timestamp = unlock_timestamp;
        time_capsule.recipient_email_hash = recipient_email_hash;
        time_capsule.password_hash = password_hash;
        time_capsule.password_hint = password_hint;
        time_capsule.message_title = message_title;
        time_capsule.created_at = clock.unix_timestamp;
        time_capsule.is_claimed = false;

        emit!(TimeCapsuleCreated {
            capsule_id: time_capsule.key(),
            sender: ctx.accounts.sender.key(),
            unlock_timestamp,
            created_at: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Récupérer le message
    pub fn retrieve_message(
        ctx: Context<RetrieveMessage>, 
        password_hash: String
    ) -> Result<String> {
        let time_capsule = &ctx.accounts.time_capsule;
        let clock = Clock::get()?;

        require!(
            clock.unix_timestamp >= time_capsule.unlock_timestamp,
            TimeCapsuleError::StillLocked
        );

        require!(
            password_hash == time_capsule.password_hash,
            TimeCapsuleError::InvalidPassword
        );

        Ok(time_capsule.encrypted_message.clone())
    }

    /// Marquer comme récupérée
    pub fn mark_as_claimed(
        ctx: Context<MarkAsClaimed>,
        password_hash: String
    ) -> Result<()> {
        let time_capsule = &mut ctx.accounts.time_capsule;
        let clock = Clock::get()?;

        require!(
            clock.unix_timestamp >= time_capsule.unlock_timestamp,
            TimeCapsuleError::StillLocked
        );

        require!(
            password_hash == time_capsule.password_hash,
            TimeCapsuleError::InvalidPassword
        );

        time_capsule.is_claimed = true;

        emit!(TimeCapsuleClaimed {
            capsule_id: time_capsule.key(),
            claimed_at: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Obtenir infos publiques
    pub fn get_capsule_info(ctx: Context<GetCapsuleInfo>) -> Result<CapsuleInfo> {
        let time_capsule = &ctx.accounts.time_capsule;
        
        Ok(CapsuleInfo {
            sender: time_capsule.sender,
            unlock_timestamp: time_capsule.unlock_timestamp,
            password_hint: time_capsule.password_hint.clone(),
            message_title: time_capsule.message_title.clone(),
            created_at: time_capsule.created_at,
            is_claimed: time_capsule.is_claimed,
        })
    }

    /// ✅ NOUVELLE FONCTION : Lister toutes les capsules d'un utilisateur
    pub fn get_user_capsules(_ctx: Context<GetUserCapsules>) -> Result<Vec<UserCapsuleInfo>> {
        // Cette fonction nécessiterait une approche différente en production
        // En général, on utiliserait des comptes PDA ou des indexers externes
        Ok(vec![])
    }

    /// Mettre à jour le prix du service
    pub fn update_price(ctx: Context<UpdatePrice>, new_price: u64) -> Result<()> {
        ctx.accounts.config.price = new_price;
        Ok(())
    }
}

// ============================================================================
// STRUCTURES
// ============================================================================

#[account]
pub struct TimeCapsule {
    pub sender: Pubkey,
    pub encrypted_message: String,
    pub unlock_timestamp: i64,
    pub recipient_email_hash: String,
    pub password_hash: String,
    pub password_hint: String,
    pub message_title: String,
    pub created_at: i64,
    pub is_claimed: bool,
}

#[account]
pub struct Config {
    pub price: u64,
    pub authority: Pubkey,
    pub treasury: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CapsuleInfo {
    pub sender: Pubkey,
    pub unlock_timestamp: i64,
    pub password_hint: String,
    pub message_title: String,
    pub created_at: i64,
    pub is_claimed: bool,
}

// ✅ NOUVELLE STRUCTURE pour le listage des capsules utilisateur
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UserCapsuleInfo {
    pub capsule_id: Pubkey,
    pub unlock_timestamp: i64,
    pub message_title: String,
    pub is_claimed: bool,
}

// ============================================================================
// CONTEXTS
// ============================================================================

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 32 + 32,
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

// ✅ GESTION MÉMOIRE OPTIMISÉE : Calcul dynamique du space
#[derive(Accounts)]
#[instruction(
    encrypted_message: String,
    unlock_timestamp: i64,
    recipient_email_hash: String,
    password_hash: String,
    password_hint: String,
    message_title: String
)]
pub struct CreateTimeCapsule<'info> {
    #[account(
        init,
        payer = sender,
        space = 8 + 32 + 4 + encrypted_message.len() + 8 + 4 + 64 + 4 + 64 + 4 + password_hint.len() + 4 + message_title.len() + 8 + 1,
        constraint = encrypted_message.len() <= 5000 @ TimeCapsuleError::MessageTooLong,
        constraint = password_hint.len() <= 500 @ TimeCapsuleError::HintTooLong,
        constraint = message_title.len() <= 200 @ TimeCapsuleError::TitleTooLong,
    )]
    pub time_capsule: Account<'info, TimeCapsule>,
    
    #[account()]
    pub config: Account<'info, Config>,

    /// CHECK: Treasury address is validated against config.treasury in the instruction logic
    #[account(mut)]
    pub treasury: AccountInfo<'info>,
    
    #[account(mut)]
    pub sender: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RetrieveMessage<'info> {
    pub time_capsule: Account<'info, TimeCapsule>,
}

#[derive(Accounts)]
pub struct MarkAsClaimed<'info> {
    #[account(mut)]
    pub time_capsule: Account<'info, TimeCapsule>,
}

#[derive(Accounts)]
pub struct GetCapsuleInfo<'info> {
    pub time_capsule: Account<'info, TimeCapsule>,
}

// ✅ NOUVEAU CONTEXT pour le listage des capsules utilisateur
#[derive(Accounts)]
pub struct GetUserCapsules<'info> {
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(
        mut,
        has_one = authority @ TimeCapsuleError::UnauthorizedAccess
    )]
    pub config: Account<'info, Config>,
    pub authority: Signer<'info>,
}

// ============================================================================
// EVENTS
// ============================================================================

#[event]
pub struct TimeCapsuleCreated {
    pub capsule_id: Pubkey,
    pub sender: Pubkey,
    pub unlock_timestamp: i64,
    pub created_at: i64,
}

#[event]
pub struct TimeCapsuleClaimed {
    pub capsule_id: Pubkey,
    pub claimed_at: i64,
}

// ============================================================================
// ERREURS
// ============================================================================

#[error_code]
pub enum TimeCapsuleError {
    #[msg("Unlock date must be in the future")]
    InvalidUnlockTime,
    
    #[msg("Message is too long (max 5KB)")]
    MessageTooLong,
    
    #[msg("Hint is too long (max 500 characters)")]
    HintTooLong,
    
    #[msg("Title is too long (max 200 characters)")]
    TitleTooLong,
    
    #[msg("This time capsule is still locked")]
    StillLocked,
    
    #[msg("Incorrect password")]
    InvalidPassword,
    
    #[msg("Invalid treasury wallet")]
    InvalidTreasury,

    #[msg("Unauthorized access")]
    UnauthorizedAccess,

    #[msg("Invalid email hash (must be SHA256 64 characters)")]
    InvalidEmailHash,
    
    #[msg("Invalid password hash (must be SHA256 64 characters)")]
    InvalidPasswordHash,
}