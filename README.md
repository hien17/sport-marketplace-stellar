# Sport Marketplace DApp

**Sport Marketplace DApp** - Blockchain-Based Decentralized Sport Item Trading Platform

## Project Description

Sport Marketplace DApp is a decentralized smart contract solution built on the Stellar blockchain using Soroban SDK. It provides a secure, transparent, and permissionless platform for buying and selling sport equipment directly on the blockchain. The contract ensures that all trading activity is governed by predefined smart contract logic, eliminating the need for centralized marketplaces or intermediaries.

The system allows users to list, unlist, relist, and purchase sport items using any Stellar Asset Contract (SAC) token, including XLM. Each listing is uniquely identified, stored with full metadata, and kept in a searchable on-chain index — ensuring data persistence, auditability, and reliability across the Stellar network.

## Project Vision

Our vision is to revolutionize the sport equipment resale market by:

- **Decentralizing Trade**: Moving peer-to-peer commerce from centralized platforms to a global, distributed blockchain
- **Ensuring Ownership**: Empowering sellers to have complete control over their listings without platform lock-in
- **Guaranteeing Transparency**: Providing a permanent, tamper-proof record of every trade that anyone can audit
- **Enabling Fair Fees**: Using on-chain, configurable fee logic so platform costs are always visible and predictable
- **Building Trustless Commerce**: Creating a marketplace where transaction integrity is guaranteed by code, not by company promises

We envision a future where athletes, collectors, and sport enthusiasts can trade freely and securely — with no middlemen, no hidden fees, and no censorship.

## Key Features

### 1. **Item Listing**

- List any sport item for sale with a single function call
- Specify title, description, category, condition, price, and preferred payment token
- Automated ID generation for unique listing identification
- Persistent storage on the Stellar blockchain

### 2. **Buy & Sell**

- Purchase any active listing directly on-chain
- Atomic token transfers: seller receives payment and platform fee is collected simultaneously
- Configurable platform fee in basis points (e.g. 250 = 2.5%)
- Protection against self-purchase and inactive listing purchases

### 3. **Listing Management**

- Unlist an active item to temporarily remove it from the marketplace
- Relist a previously unlisted item, optionally with a new price
- Update the price of any active listing at any time
- Full lifecycle tracking: `Active` → `Unlisted` / `Sold`

### 4. **On-Chain Search & Filtering**

- Fetch all listings or filter by status, seller address, or sport category
- Lightweight index stored in instance storage for efficient queries
- Full listing details retrievable individually by ID
- Real-time synchronization with the blockchain state

### 5. **Admin & Fee Management**

- Admin-controlled platform fee configuration (in basis points)
- Dedicated fee vault address to collect platform revenue
- Fee and vault address are updatable by the admin at any time
- Zero-fee mode supported by setting `fee_bps = 0`

### 6. **Stellar Network Integration**

- Leverages the high speed and low cost of Stellar
- Built using the modern Soroban Smart Contract SDK
- Supports any SAC-compatible token as payment currency
- Scalable architecture for growing item collections

## Contract Details

- Contract Address: https://stellar.expert/explorer/testnet/contract/CAJH6FQ5T7DFKHH7XHPQRIEBLL3SAOM6BSBNHUIEIVF7KFHFPCTWWP6F
- Network: Stellar Soroban (Testnet / Mainnet)
- Language: Rust + Soroban SDK

## Sport Categories Supported

| Category    | Category    |
|-------------|-------------|
| Football    | Swimming    |
| Basketball  | Cycling     |
| Tennis      | Running     |
| Baseball    | Golf        |
| Other       |             |

## Item Conditions Supported

`New` · `LikeNew` · `Good` · `Fair` · `Poor`

## Future Scope

### Short-Term Enhancements

1. **Offer System**: Allow buyers to submit offers below asking price for seller review
2. **Auction Mode**: Timed bidding auctions with automatic winner selection at expiry
3. **Image Attachment**: Link IPFS-hosted images to listings for visual browsing
4. **Search by Price Range**: Filter active listings within a minimum and maximum price band

### Medium-Term Development

5. **Escrow & Dispute Resolution**: Hold funds in escrow until the buyer confirms delivery
   - Multi-signature release of funds
   - Admin-mediated dispute resolution
   - Automatic release after a configurable timeout
6. **Reputation System**: On-chain buyer and seller ratings accumulated over trade history
7. **Bundle Listings**: Group multiple items into a single listing for discounted sets
8. **Inter-Contract Integration**: Allow other Soroban contracts to query and interact with marketplace listings

### Long-Term Vision

9. **Cross-Chain Listings**: Extend marketplace access to other blockchain networks via bridges
10. **Decentralized UI Hosting**: Host the frontend on IPFS or similar decentralized platforms
11. **AI-Powered Pricing Suggestions**: Optional integration to recommend fair market prices based on category and condition
12. **Privacy Layers**: Implement zero-knowledge proofs for anonymous trading
13. **DAO Governance**: Community-driven protocol upgrades and fee parameter voting
14. **Identity & Verification**: Integration with decentralized identity (DID) systems to verify authentic sport brand items

### Enterprise Features

15. **Brand & Retailer Storefronts**: Verified seller profiles for official sport brands and retailers
16. **Immutable Trade Logging**: Time-stamped, auditable records of all buy/sell activity for compliance
17. **Automated Royalties**: Configurable royalty splits so original creators earn on every resale
18. **Multi-Language Support**: Expand accessibility with internationalization for global sport communities

---

## Technical Requirements

- Soroban SDK (v21+)
- Rust programming language
- Stellar blockchain network (Testnet / Mainnet)

## Getting Started

Deploy the smart contract to Stellar's Soroban network and interact with it using the main functions:

- `initialize()` — Deploy and configure the marketplace with an admin, fee rate, and fee vault
- `list_item()` — Create a new listing with title, category, condition, price, and token
- `unlist_item()` — Temporarily remove a listing from the marketplace
- `relist_item()` — Reactivate an unlisted item, optionally with a new price
- `update_price()` — Change the price of an active listing
- `buy_item()` — Purchase an active listing; payment is transferred atomically on-chain
- `get_listing()` — Retrieve full details of a specific listing by ID
- `get_active_listings()` — Fetch all currently active listings
- `get_seller_listings()` — Fetch all listings by a specific seller address
- `get_listings_by_category()` — Filter active listings by sport category

---

**Sport Marketplace DApp** - Trade Your Gear, On-Chain and On Your Terms