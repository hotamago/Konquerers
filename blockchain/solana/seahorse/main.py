# Built with Seahorse v0.2.0

from seahorse.prelude import *

# This is your program's public key and it will update
# automatically when you build the project.
declare_id('CZo5rb2tkDRuukpqT8b2iKLLyXoJbAfioEtiHvNQfvR9')

class Certificate(Account):
    creator: Pubkey # 32 bytes
    owner: Pubkey # 32 bytes
    price: u64
    weight_carbon: u64
    no_u16_32_array: Array[u16, 32] # 64 bytes
    url_u16_128_array: Array[u16, 128] # 256 bytes
    # Total: 400 bytes

@instruction
def init_certificate(
    payer: Signer,
    creator: Signer,
    certificate: Empty[Certificate],
    no_sha256: u128,
    no_u16_32_array: Array[u16, 32],
    url_u16_128_array: Array[u16, 128],
    weight_carbon: u64,
    price: u64
):
    certificate = certificate.init(payer = payer, seeds = [creator, "certificate", no_sha256])
    certificate.creator = creator.key()
    certificate.owner = creator.key()
    certificate.price = price
    certificate.weight_carbon = weight_carbon
    certificate.no_u16_32_array = no_u16_32_array
    certificate.url_u16_128_array = url_u16_128_array

@instruction
def update_price_certificate(
    payer: Signer,
    owner: Signer,
    certificate: Certificate,
    price: u64
):
    assert owner.key() == certificate.owner, "Owner much be signer"
    certificate.price = price

@instruction
def trade_certificate(
    payer: Signer,
    old_owner: Signer,
    new_owner: Signer,
    certificate: Certificate
):
    assert old_owner.key() == certificate.owner, "Owner much be signer"
    assert new_owner.key() != certificate.owner, "New owner must be different"

    # Check if user have enough money
    # Transfer money to owner
    new_owner.transfer_lamports(old_owner, certificate.price)

    # Transfer certificate to new owner
    certificate.owner = new_owner.key()