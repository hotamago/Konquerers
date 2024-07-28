#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

pub mod dot;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

use dot::program::*;
use std::{cell::RefCell, rc::Rc};

declare_id!("CZo5rb2tkDRuukpqT8b2iKLLyXoJbAfioEtiHvNQfvR9");

pub mod seahorse_util {
    use super::*;

    #[cfg(feature = "pyth-sdk-solana")]
    pub use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};
    use std::{collections::HashMap, fmt::Debug, ops::Deref};

    pub struct Mutable<T>(Rc<RefCell<T>>);

    impl<T> Mutable<T> {
        pub fn new(obj: T) -> Self {
            Self(Rc::new(RefCell::new(obj)))
        }
    }

    impl<T> Clone for Mutable<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Deref for Mutable<T> {
        type Target = Rc<RefCell<T>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Debug> Debug for Mutable<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl<T: Default> Default for Mutable<T> {
        fn default() -> Self {
            Self::new(T::default())
        }
    }

    impl<T: Clone> Mutable<Vec<T>> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    impl<T: Clone, const N: usize> Mutable<[T; N]> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    #[derive(Clone)]
    pub struct Empty<T: Clone> {
        pub account: T,
        pub bump: Option<u8>,
    }

    #[derive(Clone, Debug)]
    pub struct ProgramsMap<'info>(pub HashMap<&'static str, AccountInfo<'info>>);

    impl<'info> ProgramsMap<'info> {
        pub fn get(&self, name: &'static str) -> AccountInfo<'info> {
            self.0.get(name).unwrap().clone()
        }
    }

    #[derive(Clone, Debug)]
    pub struct WithPrograms<'info, 'entrypoint, A> {
        pub account: &'entrypoint A,
        pub programs: &'entrypoint ProgramsMap<'info>,
    }

    impl<'info, 'entrypoint, A> Deref for WithPrograms<'info, 'entrypoint, A> {
        type Target = A;

        fn deref(&self) -> &Self::Target {
            &self.account
        }
    }

    pub type SeahorseAccount<'info, 'entrypoint, A> =
        WithPrograms<'info, 'entrypoint, Box<Account<'info, A>>>;

    pub type SeahorseSigner<'info, 'entrypoint> = WithPrograms<'info, 'entrypoint, Signer<'info>>;

    #[derive(Clone, Debug)]
    pub struct CpiAccount<'info> {
        #[doc = "CHECK: CpiAccounts temporarily store AccountInfos."]
        pub account_info: AccountInfo<'info>,
        pub is_writable: bool,
        pub is_signer: bool,
        pub seeds: Option<Vec<Vec<u8>>>,
    }

    #[macro_export]
    macro_rules! seahorse_const {
        ($ name : ident , $ value : expr) => {
            macro_rules! $name {
                () => {
                    $value
                };
            }

            pub(crate) use $name;
        };
    }

    #[macro_export]
    macro_rules! assign {
        ($ lval : expr , $ rval : expr) => {{
            let temp = $rval;

            $lval = temp;
        }};
    }

    #[macro_export]
    macro_rules! index_assign {
        ($ lval : expr , $ idx : expr , $ rval : expr) => {
            let temp_rval = $rval;
            let temp_idx = $idx;

            $lval[temp_idx] = temp_rval;
        };
    }

    pub(crate) use assign;

    pub(crate) use index_assign;

    pub(crate) use seahorse_const;
}

#[program]
mod compi {
    use super::*;
    use seahorse_util::*;
    use std::collections::HashMap;

    #[derive(Accounts)]
    # [instruction (no_sha256 : u128 , no: [u16; 32] , url: [u16; 128] , weight_carbon : u64 , price : u64)]
    pub struct InitCertificate<'info> {
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(mut)]
        pub creator: Signer<'info>,
        # [account (init , space = std :: mem :: size_of :: < dot :: program :: Certificate > () + 8 , payer = payer , seeds = [creator . key () . as_ref () , "certificate" . as_bytes () . as_ref () , no_sha256 . to_le_bytes () . as_ref ()] , bump)]
        pub certificate: Box<Account<'info, dot::program::Certificate>>,
        pub rent: Sysvar<'info, Rent>,
        pub system_program: Program<'info, System>,
    }

    pub fn init_certificate(
        ctx: Context<InitCertificate>,
        no_sha256: u128,
        no: [u16; 32],
        url: [u16; 128],
        weight_carbon: u64,
        price: u64,
    ) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let payer = SeahorseSigner {
            account: &ctx.accounts.payer,
            programs: &programs_map,
        };

        let creator = SeahorseSigner {
            account: &ctx.accounts.creator,
            programs: &programs_map,
        };

        let certificate = Empty {
            account: dot::program::Certificate::load(&mut ctx.accounts.certificate, &programs_map),
            bump: Some(ctx.bumps.certificate),
        };

        init_certificate_handler(
            payer.clone(),
            creator.clone(),
            certificate.clone(),
            no_sha256,
            no,
            url,
            weight_carbon,
            price,
        );

        dot::program::Certificate::store(certificate.account);

        return Ok(());
    }

    #[derive(Accounts)]
    pub struct TradeCertificate<'info> {
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(mut)]
        pub old_owner: Signer<'info>,
        #[account(mut)]
        pub new_owner: Signer<'info>,
        #[account(mut)]
        pub certificate: Box<Account<'info, dot::program::Certificate>>,
        pub system_program: Program<'info, System>,
    }

    pub fn trade_certificate(ctx: Context<TradeCertificate>) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let payer = SeahorseSigner {
            account: &ctx.accounts.payer,
            programs: &programs_map,
        };

        let old_owner = SeahorseSigner {
            account: &ctx.accounts.old_owner,
            programs: &programs_map,
        };

        let new_owner = SeahorseSigner {
            account: &ctx.accounts.new_owner,
            programs: &programs_map,
        };

        let certificate =
            dot::program::Certificate::load(&mut ctx.accounts.certificate, &programs_map);

        trade_certificate_handler(
            payer.clone(),
            old_owner.clone(),
            new_owner.clone(),
            certificate.clone(),
        );

        dot::program::Certificate::store(certificate);

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (price : u64)]
    pub struct UpdatePriceCertificate<'info> {
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(mut)]
        pub owner: Signer<'info>,
        #[account(mut)]
        pub certificate: Box<Account<'info, dot::program::Certificate>>,
    }

    pub fn update_price_certificate(
        ctx: Context<UpdatePriceCertificate>,
        price: u64,
    ) -> Result<()> {
        let mut programs = HashMap::new();
        let programs_map = ProgramsMap(programs);
        let payer = SeahorseSigner {
            account: &ctx.accounts.payer,
            programs: &programs_map,
        };

        let owner = SeahorseSigner {
            account: &ctx.accounts.owner,
            programs: &programs_map,
        };

        let certificate =
            dot::program::Certificate::load(&mut ctx.accounts.certificate, &programs_map);

        update_price_certificate_handler(payer.clone(), owner.clone(), certificate.clone(), price);

        dot::program::Certificate::store(certificate);

        return Ok(());
    }
}
