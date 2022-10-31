# Transaction Server

## Version: 0.0.1

Background
---------------------------------

In order to facilitate the preparation of on chain transactions that can be signed and submitted in coordination with point of sale and ecommerce transactions, a separate transaction server is maintained. The transaction server is accessed via end points that conform to the Solana Pay [Transaction Request Standard](https://github.com/solana-labs/solana-pay/blob/master/SPEC.md#specification-transaction-request). It is designed to be accessible by multiple merchants and their customers without requiring either to store their keys on the server.

Below are basic transaction types that the transaction server needs to facilitate:
1. Create merchant group
2. Update group members (TODO)
3. Create promo token mint with metadata
4. Update promo token (TODO)
5. Merchant mint promo token to customer
6. Customer delegate promo token to merchant
7. Merchant burn delegated token
8. Customer undelegate token from merchant (TODO)
9. Customer transfer token (TODO)

The transaction flows are designed to have customers pay as little as possible to facilitate the transactions. Likewise, payment of merchant fees are centralized within merchant groups.

See [purchase flow](./token_metadata.md#purchase-flow) for a diagram of the flow of transactions between a merchant and customer in a typical point of sale purchase scenario. This flow requires a customer to delegate a token to a merchant in order to apply a discount to an order. The merchant then burns the delegated token when payment for the order is completed. This enables the customer to undelegate the token without requiring additional approval from the merchant if for whatever reason the order is abandoned or the discount ends up not being applied.

Transaction Signing Requirements
---------------------------------

| Transaction          | Group Owner | Group Member | Token Owner |
| -------------------- |:-----------:|:------------:|:-----------:|
| Create Group         |  [x]        |              |             |
| Create Promo         |  [x]        |              |             |
| Mint Promo Token     |             |  [x]         |  [x]        |
| Delegate Token       |             |              |  [x]        |
| Burn Delegated Token |             |  [x]         |             |


Create Group
---------------------------------

```
/promo/group/{groupSeed}/{members}/{lamports}/{memo}
```
In order to enable merchants with multiple locations and/or multiple check out stations within a single location to create and manage their on chain offers, each promo token mint is owned by a group. A group consists of an owner, prepresented by a Pubkey, and members, represented by a vector of Pubkeys.

The owner of the group can add and remove members. Any of the members can initiate mint, delegate or burn transactions. The group is the payer of the bokoup token mint creation and burn fees.

Anybody can create a group, with the payer of the transaction to create the group designated as the owner of the group. The group account is owned by the bokoup program with constraints on its use enforced by the program. The account address is program derived based on a supplied Pubkey.

To start with, the intial point of sale and ecommerce applications support only one group per organization.

### Methods

1. `GET` request returns logo and label identifying the application
2. `POST` with payer/owner address in body returns transaction to create group
3. Owner signs and submits transaction directly to the network

### Parameters

* `{groupSeed}` base58 encoded string representation of a Pubkey
* `{members}` url encoded array of base58 string representations of Pubkeys
* `{lamports}` is the amount to be transferred from payer/owner to the group to pay for transaction fees incurred by members of the group
* `{memo}` Optional url encoded string to be included as a memo in the on chain transaction to create the group. If a json encoded string, will be available from the bokoup graphql data api as json.


Create Promo Token
---------------------------------

```
/promo/create
```
This endpoint is used to create a turnkey promo token, including uploading the associated image and metadata json to Arweave for permantent storage. It accepts a multipart stream with the following fields in the order presented:

1. `GET` request returns logo and application id
2. `POST` with payer/owner address in body returns transaction to create promo
3. Owner signs and submits transaction directly to the network

### metadata
Expected to be a field with a name of `metadata` containing a string, representing the json as described in the [Token Metadata Specifications](token_metadata.md), excluding the `image` and `files` keys, which will be updated automatically based on ids of the associated Arweave upload transactions.

### image
Expected to be a field named `image` containing the bytes of the image and the correct mime type.

### groupSeed
Expected to be a field with a name of `groupSeed` containing a base58 encoded string representation of the Pubkey used to create the group that will own the promo. The group must already exist or the transaction will fail.

### memo
Optional field with a field with a name of `memo` containing a string to be included as a memo in the on chain transaction to create the promo. If a json encoded string, will be available from the bokoup graphql data api as json.


Mint Promo Token
---------------------------------

Merchants may want to have tokens that are freely mintable to anyone who wants one. They may also want to have other tokens that require their approval/signature to be minted.

### Freely Mintable
To make tokens freely available to be minted without merchant approval, merchants can add the designated platform as a member of their group and use following url for minting tokens, likely most often by creating QR codes encoded with parameterized versions of it.

An example use case is placing low value promotional offers in printed QR codes on in store displays or publicly accessible websites that prospective customers can scan and receive without requiring any additional approval.

```
/promo/mint/{mintString}/{message}/{memo}
```
#### Methods
1. `GET` request returns logo and label identifying the application
2. `POST` with token owner address in body returns transaction and message
3. Token owner signs and submits transaction directly to the network

#### Parameters
* `{mintString}` base58 encoded string representation of Pubkey address of mint associated with promo
* `{message}` url encoded string to be displayed in the receiving application to describe the received transaction
* `{memo}` Optional url encoded string to be included as a memo in the on chain transaction to create the group. If a json encoded string, will be available from the bokoup graphql data api as json.

### Merchant Approval/Signature Required
Merchant approval to mint a token is achieved via the following steps:
1. Merchant requests mint transaction from server
2. Merchant signs and posts transaction back to server
3. Server stores transaction signed by merchant
4. Customer requests transaction
5. Customer signs transaction and submits to the network

An example use case is a higher value promotion granted at the point of sale upon completion of a qualifying purchase.

#### Merchant transaction request
```
/promo/merchant/mint/{mintString}/{tokenOwner}/{message}/{memo}
```
##### Methods
1. `GET` request returns logo and label identifying the application
2. `POST` with merchant address in body returns transaction to mint token and message. Merchant address can be any member of group associated with promo token.
3. Merchant signs transaction and posts back to server

##### Parameters
* `{mintString}` base58 encoded string representation of Pubkey address of mint associated with promo
* `{tokenOwner}` base58 encoded string representation of Pubkey address to which the token will be minted. Should be the owner address, not the token acount address. An associated token account will be created if one doesn't already exist.
* `{message}` url encoded string to be displayed in the receiving application to describe the transaction to the merchant
* `{memo}` Optional url encoded string to be included as a memo in the on chain transaction to mint the token. If a json encoded string, will be available from the bokoup graphql data api as json.

#### Merchant post signed transaction
```
/promo/merchant/signed
```
##### Methods
1. `POST` with json object including transaction as base64 encoded string and customer message as utf-8 string

#### Customer transaction request
```
/promo/signed/{signature}/{message}
```
1. `GET` request returns logo and label identifying the application
2. `POST` with customer / token owner address in body returns transaction and message
3. Token owner signs and submits transaction directly to the network

##### Parameters
* `{message}` url encoded string to be displayed in the receiving application to describe the transaction to the customer

### Implementation Details
* The platform queries the bokoup data api to confirm that the platform address is included in the members of the group that owns the promo, returning a bad request response if not included.
* The process relies on the requirement that the transaction be signed by both parties and submitted to the network within 150 blocks of the blockhash included with the first signature. In practice, this provides a window of approximately one minute for the token owner to sign the transaction after the merchant does, which should be ample in the context of a typical point of sale or ecommerce transaction.
* The json object with the transaction and customer message gets stored in an in memory key-value store with using the signature as the key, set to expire after one minute.


## Delegate Promo Token
---------------------------------

Merchants can initiate a request to have a customer delegate a token to them by following
a process similar to the one described above for minting tokens with merchant approval.

1. Merchant requests delegate transaction from server
2. Merchant signs and posts transaction back to server
3. Server stores transaction signed by merchant
4. Customer requests transaction
5. Customer signs transaction and submits to the network

#### Merchant transaction request
```
/promo/merchant/delegate/{mintString}/{tokenOwner}/{message}/{memo}
```
##### Methods
1. `GET` request returns logo and label identifying the application
2. `POST` with merchant address in body returns transaction to delegate token and message. Merchant address can be any member of group associated with promo token.
3. Merchant signs transaction and posts back to server

##### Parameters
* `{mintString}` base58 encoded string representation of Pubkey address of mint associated with promo
* `{tokenOwner}` base58 encoded string representation of Pubkey address ok token owner. Should be the owner address, not the token acount address.
* `{message}` url encoded string to be displayed in the receiving application to describe the transaction to the merchant.
* `{memo}` Optional url encoded string to be included as a memo in the on chain transaction to delegate the token. If a json encoded string, will be available from the bokoup graphql data api as json.

#### Merchant post signed transaction
```
/promo/merchant/signed
```
##### Methods
1. `POST` with json object including transaction as base64 encoded string and customer message as utf-8 string

#### Customer transaction request
```
/promo/signed/{signature}/{message}
```
1. `GET` request returns logo and label identifying the application
2. `POST` with customer / token owner address in body returns transaction and message
3. Token owner signs and submits transaction directly to the network

##### Parameters
* `{message}` url encoded string to be displayed in the receiving application to describe the transaction to the customer

### Implementation Details
* The platform queries the bokoup data api to confirm that the platform address is included in the members of the group that owns the promo, returning a bad request response if not included.
* The process relies on the requirement that the transaction be signed by both parties and submitted to the network within 150 blocks of the blockhash included with the first signature. In practice, this provides a window of approximately one minute for the token owner to sign the transaction after the merchant does, which should be ample in the context of a typical point of sale or ecommerce transaction.
* The json object with the transaction and customer message gets stored in an in memory key-value store with using the signature as the key, set to expire after one minute.