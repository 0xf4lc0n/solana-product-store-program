use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum ProductInstruction {
    AddProduct { name: String, price: f64 },
    UpdateProduct { name: String },
    UpdatePrice { price: f64 },
}

#[derive(BorshDeserialize)]
struct AddProductPayload {
    name: String,
    price: f64,
}

#[derive(BorshDeserialize)]
struct UpdateProductPayload {
    name: String,
}

#[derive(BorshDeserialize)]
struct UpdatePricePayload {
    price: f64,
}

impl ProductInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        let instruction = match variant {
            0 => {
                let payload = AddProductPayload::try_from_slice(rest).unwrap();
                Self::AddProduct {
                    name: payload.name,
                    price: payload.price,
                }
            }
            1 => {
                let payload = UpdateProductPayload::try_from_slice(rest).unwrap();
                Self::UpdateProduct { name: payload.name }
            }
            2 => {
                let payload = UpdatePricePayload::try_from_slice(rest).unwrap();
                Self::UpdatePrice {
                    price: payload.price,
                }
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        };

        return Ok(instruction);
    }
}
