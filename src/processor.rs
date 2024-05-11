use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh0_9::try_from_slice_unchecked,
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    instruction::ProductInstruction,
    state::{ProductAccountState, ProductPrice, ProductPriceCounter},
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = ProductInstruction::unpack(instruction_data)?;

    match instruction {
        ProductInstruction::AddProduct { name, price, id } => {
            add_product(program_id, accounts, id, name, price)
        }
        ProductInstruction::UpdateProduct { name } => update_product(program_id, accounts, name),
        ProductInstruction::UpdatePrice { price } => add_price(program_id, accounts, price),
    }
}

pub fn add_product(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: u64,
    name: String,
    price: f64,
) -> ProgramResult {
    msg!("Adding product...");
    msg!("Id: {}", id);
    msg!("Name: {}", name);
    msg!("Price: {:.2}", price);

    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;
    let pda_product = next_account_info(account_info_iter)?;
    let pda_counter = next_account_info(account_info_iter)?;
    let _pda_price = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (pda, bump_seed) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), id.to_be_bytes().as_ref()],
        program_id,
    );
    if pda != *pda_product.key {
        msg!("Invalid seeds for PDA");
        return Err(ProgramError::InvalidArgument);
    }

    if name.len() == 0 || price <= 0.0 {
        msg!("Name cannot be empty, price cannot be negative");
        // TODO: add custom error
        return Err(ProgramError::InvalidArgument);
    }

    if ProductAccountState::get_account_size(name.clone()) > 1000 {
        msg!("Data length is larger than 1000 bytes");
        // TODO: add custom error
        return Err(ProgramError::InvalidArgument);
    }

    let account_size: usize = 1000;

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_size);

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_product.key,
            rent_lamports,
            account_size.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            pda_product.clone(),
            system_program.clone(),
        ],
        &[&[
            initializer.key.as_ref(),
            id.to_be_bytes().as_ref(),
            &[bump_seed],
        ]],
    )?;

    msg!("Created Product PDA: {}", pda);

    msg!("Unpacking state account");
    let mut product_account_data =
        try_from_slice_unchecked::<ProductAccountState>(&pda_product.data.borrow()).unwrap();
    msg!("State account borrowed");

    msg!("Checking if Product Account is already initialized");
    if product_account_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    product_account_data.discriminator = ProductAccountState::DISCRIMINATOR.to_string();
    product_account_data.seller = *initializer.key;
    product_account_data.id = id;
    product_account_data.name = name;
    product_account_data.price = price;
    product_account_data.is_initialized = true;

    msg!("Serializing account");
    product_account_data.serialize(&mut &mut pda_product.data.borrow_mut()[..])?;
    msg!("State account serialized");

    msg!("Creating Price Counter");
    let rent = Rent::get()?;
    let counter_rent_lamports = rent.minimum_balance(ProductPriceCounter::SIZE);

    let (counter, counter_bump) =
        Pubkey::find_program_address(&[pda.as_ref(), "price".as_ref()], program_id);

    if counter != *pda_counter.key {
        msg!("Invalid seeds for Counter PDA");
        return Err(ProgramError::InvalidArgument);
    }

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_counter.key,
            counter_rent_lamports,
            ProductPriceCounter::SIZE.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            pda_counter.clone(),
            system_program.clone(),
        ],
        &[&[pda.as_ref(), "price".as_ref(), &[counter_bump]]],
    )?;
    msg!("Price Counter created");

    let mut counter_data =
        try_from_slice_unchecked::<ProductPriceCounter>(&pda_counter.data.borrow()).unwrap();

    msg!("Checking if Price Counter account is already initialized");
    if counter_data.is_initialized {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    counter_data.discriminator = ProductPriceCounter::DISCRIMINATOR.to_string();
    counter_data.counter = 0;
    counter_data.is_initialized = true;

    msg!("Price count: {}", counter_data.counter);
    counter_data.serialize(&mut &mut pda_counter.data.borrow_mut()[..])?;

    // Adding price
    add_price(program_id, accounts, price)?;

    Ok(())
}

pub fn update_product(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
) -> ProgramResult {
    msg!("Updating product...");
    msg!("Name: {}", name);

    let account_info_iter = &mut accounts.iter();

    let seller = next_account_info(account_info_iter)?;
    let pda_product = next_account_info(account_info_iter)?;

    if pda_product.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    if !seller.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!("Unpacking state account");
    let mut account_data =
        try_from_slice_unchecked::<ProductAccountState>(&pda_product.data.borrow()).unwrap();

    msg!("Product name: {}", account_data.name);

    let (pda, _bump_seed) = Pubkey::find_program_address(
        &[seller.key.as_ref(), account_data.id.to_be_bytes().as_ref()],
        program_id,
    );

    if pda != *pda_product.key {
        msg!("Invalid seeds for PDA");
        // TODO: Custom error
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Checking if movie account is initialized");
    if !account_data.is_initialized() {
        msg!("Account is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    if name.len() == 0 {
        msg!("Name cannot be empty");
        // TODO: add custom error
        return Err(ProgramError::InvalidArgument);
    }

    if ProductAccountState::get_account_size(name.clone()) > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Product before update:");
    msg!("Name: {}", account_data.name);

    account_data.name = name;

    msg!("Product after update:");
    msg!("Name: {}", account_data.name);

    msg!("Serializing account");
    account_data.serialize(&mut &mut pda_product.data.borrow_mut()[..])?;
    msg!("State account serialized");

    Ok(())
}

pub fn add_price(program_id: &Pubkey, accounts: &[AccountInfo], price: f64) -> ProgramResult {
    msg!("Adding price...");
    let account_info_inter = &mut accounts.iter();

    let product_owner = next_account_info(account_info_inter)?;
    let pda_product = next_account_info(account_info_inter)?;
    let pda_counter = next_account_info(account_info_inter)?;
    let pda_price = next_account_info(account_info_inter)?;
    let system_program = next_account_info(account_info_inter)?;

    let mut counter_data =
        try_from_slice_unchecked::<ProductPriceCounter>(&pda_counter.data.borrow()).unwrap();

    let account_len = ProductPrice::SIZE;

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    let (pda, bump_seed) = Pubkey::find_program_address(
        &[
            pda_product.key.as_ref(),
            counter_data.counter.to_be_bytes().as_ref(),
        ],
        program_id,
    );

    if pda != *pda_price.key {
        msg!("Invalid seeds for PDA");
        // TODO: Add custom error
        return Err(ProgramError::InvalidArgument);
    }

    invoke_signed(
        &system_instruction::create_account(
            product_owner.key,
            pda_price.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            product_owner.clone(),
            pda_price.clone(),
            system_program.clone(),
        ],
        &[&[
            pda_product.key.as_ref(),
            counter_data.counter.to_be_bytes().as_ref(),
            &[bump_seed],
        ]],
    )?;

    msg!("Created Price account");
    let mut price_data =
        try_from_slice_unchecked::<ProductPrice>(&pda_price.data.borrow()).unwrap();

    msg!("Checking if price account is already initialized");
    if price_data.is_initialized() {
        msg!("Price account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp as u64;

    price_data.discriminator = ProductPrice::DISCRIMINATOR.to_string();
    price_data.price = price;
    price_data.product = *pda_product.key;
    price_data.is_initialized = true;
    //price_data.timestamp = current_timestamp;

    price_data.serialize(&mut &mut pda_price.data.borrow_mut()[..])?;

    msg!("Price count: {}", counter_data.counter);
    counter_data.counter += 1;
    counter_data.serialize(&mut &mut pda_counter.data.borrow_mut()[..])?;

    msg!("Updating price in the Product Account");
    msg!("Unpacking state account");
    let mut account_data =
        try_from_slice_unchecked::<ProductAccountState>(&pda_product.data.borrow()).unwrap();

    let (pda, _bump_seed) = Pubkey::find_program_address(
        &[
            product_owner.key.as_ref(),
            account_data.id.to_be_bytes().as_ref(),
        ],
        program_id,
    );

    if pda != *pda_product.key {
        msg!("Invalid seeds for Product PDA");
        // TODO: Custom error
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Checking if Product Account is initialized");
    if !account_data.is_initialized() {
        msg!("Account is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    if price <= 0.0 {
        msg!("Price cannot be lower than 0.0");
        // TODO: add custom error
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Product before update:");
    msg!("Price: {}", account_data.price);

    account_data.price = price;

    msg!("Product after update:");
    msg!("Price: {}", account_data.price);

    msg!("Serializing account");
    account_data.serialize(&mut &mut pda_product.data.borrow_mut()[..])?;
    msg!("State account serialized");

    Ok(())
}
