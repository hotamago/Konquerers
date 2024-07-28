#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use crate::{id, seahorse_util::*};
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::{cell::RefCell, rc::Rc};

#[account]
#[derive(Debug)]
pub struct Certificate {
    pub creator: Pubkey,
    pub owner: Pubkey,
    pub price: u64,
    pub weight_carbon: u64,
    pub no: [u16; 32],
    pub url: [u16; 128],
}

impl<'info, 'entrypoint> Certificate {
    pub fn load(
        account: &'entrypoint mut Box<Account<'info, Self>>,
        programs_map: &'entrypoint ProgramsMap<'info>,
    ) -> Mutable<LoadedCertificate<'info, 'entrypoint>> {
        let creator = account.creator.clone();
        let owner = account.owner.clone();
        let price = account.price;
        let weight_carbon = account.weight_carbon;
        let no = Mutable::new(account.no.clone());
        let url = Mutable::new(account.url.clone());

        Mutable::new(LoadedCertificate {
            __account__: account,
            __programs__: programs_map,
            creator,
            owner,
            price,
            weight_carbon,
            no,
            url,
        })
    }

    pub fn store(loaded: Mutable<LoadedCertificate>) {
        let mut loaded = loaded.borrow_mut();
        let creator = loaded.creator.clone();

        loaded.__account__.creator = creator;

        let owner = loaded.owner.clone();

        loaded.__account__.owner = owner;

        let price = loaded.price;

        loaded.__account__.price = price;

        let weight_carbon = loaded.weight_carbon;

        loaded.__account__.weight_carbon = weight_carbon;

        let no = loaded.no.borrow().clone();

        loaded.__account__.no = no;

        let url = loaded.url.borrow().clone();

        loaded.__account__.url = url;
    }
}

#[derive(Debug)]
pub struct LoadedCertificate<'info, 'entrypoint> {
    pub __account__: &'entrypoint mut Box<Account<'info, Certificate>>,
    pub __programs__: &'entrypoint ProgramsMap<'info>,
    pub creator: Pubkey,
    pub owner: Pubkey,
    pub price: u64,
    pub weight_carbon: u64,
    pub no: Mutable<[u16; 32]>,
    pub url: Mutable<[u16; 128]>,
}

pub fn init_certificate_handler<'info>(
    mut payer: SeahorseSigner<'info, '_>,
    mut creator: SeahorseSigner<'info, '_>,
    mut certificate: Empty<Mutable<LoadedCertificate<'info, '_>>>,
    mut no_sha256: u128,
    mut no: [u16; 32],
    mut url: [u16; 128],
    mut weight_carbon: u64,
    mut price: u64,
) -> () {
    let mut certificate = certificate.account.clone();

    assign!(certificate.borrow_mut().creator, creator.key());

    assign!(certificate.borrow_mut().owner, creator.key());

    assign!(certificate.borrow_mut().price, price);

    assign!(certificate.borrow_mut().weight_carbon, weight_carbon);

    assign!(certificate.borrow_mut().no, Mutable::<[u16; 32]>::new(no));

    assign!(certificate.borrow_mut().url, Mutable::<[u16; 128]>::new(url));
}

pub fn trade_certificate_handler<'info>(
    mut payer: SeahorseSigner<'info, '_>,
    mut old_owner: SeahorseSigner<'info, '_>,
    mut new_owner: SeahorseSigner<'info, '_>,
    mut certificate: Mutable<LoadedCertificate<'info, '_>>,
) -> () {
    if !(old_owner.key() == certificate.borrow().owner) {
        panic!("Owner much be signer");
    }

    if !(new_owner.key() != certificate.borrow().owner) {
        panic!("New owner must be different");
    }

    solana_program::program::invoke(
        &solana_program::system_instruction::transfer(
            &new_owner.key(),
            &old_owner.clone().key(),
            certificate.borrow().price.clone(),
        ),
        &[
            new_owner.to_account_info(),
            old_owner.clone().to_account_info(),
            new_owner.programs.get("system_program").clone(),
        ],
    )
    .unwrap();

    assign!(certificate.borrow_mut().owner, new_owner.key());
}

pub fn update_price_certificate_handler<'info>(
    mut payer: SeahorseSigner<'info, '_>,
    mut owner: SeahorseSigner<'info, '_>,
    mut certificate: Mutable<LoadedCertificate<'info, '_>>,
    mut price: u64,
) -> () {
    if !(owner.key() == certificate.borrow().owner) {
        panic!("Owner much be signer");
    }

    assign!(certificate.borrow_mut().price, price);
}
