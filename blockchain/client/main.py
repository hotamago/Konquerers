from typing import Optional
import uvicorn

from fastapi import FastAPI, Body, Depends, HTTPException,  File, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from config import *
from pydantic import BaseModel
import json
import hashlib

# import data
from hotaSolana.hotaSolanaDataBase import *
from hotaSolana.hotaSolanaData import *
from hotaSolana.hotaSolanaMeathod import *
from hotaSolana.bs58 import bs58

from baseAPI import *

description = """
hotaSolana API helps you do awesome stuff. 🚀
"""

app = FastAPI(title="Solana API",
              description=description,
              summary="This is a Solana API",
              version="v2.0",
              contact={
                  "name": "Hotamago Master",
                  "url": "https://www.linkedin.com/in/hotamago/",
              })

origins = ["*"]

app.add_middleware(
    CORSMiddleware,
    allow_origins=origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Solana Client
client = HotaSolanaRPC(programId, False, "devnet")

# Solana instruction data
@BaseStructClass
class Certificate:
    creator=HotaPublicKey()
    owner=HotaPublicKey()
    price=HotaUint64(0)
    weight_carbon=HotaUint64(0)
    no=HotaStringUTF16(32, "")
    url=HotaStringUTF16(128, "")

##### Router
@BaseInstructionDataClass("init_certificate")
class InitCertificateInstruction:
    no_sha256=HotaUint128(0)
    no=HotaStringUTF16(32, "")
    url=HotaStringUTF16(128, "")
    weight_carbon=HotaUint64(0)
    price=HotaUint64(0)

class InitCertificateModel(BaseModel):
    owner_private_key: str
    no: str
    url: str
    weight_carbon: int
    price: int

@app.post("/init-certificate")
async def init_certificate(data: InitCertificateModel):
    def fun():
        owner_keypair = makeKeyPair(data.owner_private_key)

        instruction_data = InitCertificateInstruction()
        no_sha256: bytes = hash256(data.no)
        # Get first 16 bytes
        instruction_data.get("no_sha256").deserialize(no_sha256[:16])
        instruction_data.get("no").object2struct(data.no)
        instruction_data.get("url").object2struct(data.url)
        instruction_data.get("weight_carbon").object2struct(data.weight_carbon)
        instruction_data.get("price").object2struct(data.price)

        certificate_pubkey = findProgramAddress(createBytesFromArrayBytes(
            owner_keypair.public_key.byte_value,
            "certificate".encode("utf-8"),
            bytes(instruction_data.get("no_sha256").serialize())
        ),
        client.program_id)

        transaction_address = client.send_transaction(
            instruction_data,
            [
                makeKeyPair(payerPrivateKey).public_key,
                owner_keypair.public_key,
                certificate_pubkey,
                makePublicKey(sysvar_rent),
                makePublicKey(system_program),
            ],
            [
                makeKeyPair(payerPrivateKey),
                owner_keypair
            ],
            makeKeyPair(payerPrivateKey).public_key
        )

        return {
            "transaction_address": transaction_address,
            "public_key": bs58.encode(certificate_pubkey.byte_value),
        }

    return make_response_auto_catch(fun)


# Update price certificate
@BaseInstructionDataClass("update_price_certificate")
class UpdatePriceCertificateInstruction:
    price=HotaUint64(0)

class UpdatePriceCertificateModel(BaseModel):
    owner_private_key: str
    certificate_public_key: str
    price: int

@app.post("/update-price-certificate")
async def update_price_certificate(data: UpdatePriceCertificateModel):
    def fun():
        owner_keypair = makeKeyPair(data.owner_private_key)
        certificate_pubkey = makePublicKey(data.certificate_public_key)

        instruction_data = UpdatePriceCertificateInstruction()
        instruction_data.get("price").object2struct(data.price)

        transaction_address = client.send_transaction(
            instruction_data,
            [
                makeKeyPair(payerPrivateKey).public_key,
                owner_keypair.public_key,
                certificate_pubkey,
                makePublicKey(sysvar_rent),
                makePublicKey(system_program),
            ],
            [
                makeKeyPair(payerPrivateKey),
                owner_keypair
            ],
            makeKeyPair(payerPrivateKey).public_key
        )

        return {
            "transaction_address": transaction_address,
            "public_key": bs58.encode(certificate_pubkey.byte_value),
        }

    return make_response_auto_catch(fun)

# Trade certificate
@BaseInstructionDataClass("trade_certificate")
class TradeBlockInstruction:
    pass

class TradeCertificateModel(BaseModel):
    old_owner_private_key: str
    new_owner_private_key: str
    certificate_public_key: str

@app.post("/trade-certificate")
async def trade_block(data: TradeCertificateModel):
    def fun():
        owner_keypair = makeKeyPair(data.old_owner_private_key)
        buyer_keypair = makeKeyPair(data.new_owner_private_key)
        certificate_pubkey = makePublicKey(data.certificate_public_key)

        transaction_address = client.send_transaction(
            TradeBlockInstruction(),
            [
                makeKeyPair(payerPrivateKey).public_key,
                owner_keypair.public_key,
                buyer_keypair.public_key,
                certificate_pubkey,
                # makePublicKey(sysvar_rent),
                makePublicKey(system_program),
            ],
            [
                makeKeyPair(payerPrivateKey),
                owner_keypair,
                buyer_keypair
            ],
            makeKeyPair(payerPrivateKey).public_key
        )

        return {
            "transaction_address": transaction_address,
            "public_key": bs58.encode(certificate_pubkey.byte_value),
        }

    return make_response_auto_catch(fun)

### API Get
@app.get("/get-certificate-data")
async def get_certificate_data(public_key: str):
    def fun():
        res: dict = client.get_account_data(PublicKey(public_key), Certificate, [8, 0])
        return res
    return make_response_auto_catch(fun)

### Common
@app.post("/convert-keypair-to-private-key")
async def convert_keypair_to_private_key(file: UploadFile):
    # Bytes to string
    result = file.file.read()
    keypair_json = json.loads(result)
    keypair_bytes = bytes(keypair_json)
    return {
        "public_key": bs58.encode(keypair_bytes[32:]),
        "private_key": bs58.encode(keypair_bytes),
    }

@app.get("/get-info")
async def get_info(public_key: str):
    return make_response_auto_catch(lambda: client.get_account_info(PublicKey(public_key)))

@app.get("/get-balance")
async def get_balance(public_key: str):
    return make_response_auto_catch(client.get_balance(public_key))

@app.post("/airdrop")
async def airdrop(public_key: str, amount: int = 1):
    return make_response_auto_catch(client.drop_sol(public_key, amount))

# Run
if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=openPortAPI)
