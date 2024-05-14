//! Decentralized Music Royalties Distribution Platform
//!
//! This platform aims to revolutionize the way music royalties are distributed, ensuring fair compensation for artists and creators. By leveraging blockchain technology and smart contracts, the platform provides transparency, efficiency, and trust in the distribution process.

#![no_std]

use core::ops::Add;

use blockchain_sdk::storage::Persistent;
use blockchain_sdk::{
    contract, contracterror, contractimpl, contracttype, map, panic_with_error, token, vec,
    Address, Env, Map, Symbol, Vec
};

/// State of the royalties distribution
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq)]
enum RoyaltiesState {
    Initialized = 1,
    Active = 2,
    Finished = 3,
}

/// Datakey holds all possible storage keys this contract uses.
#[derive(Clone, Copy)]
#[contracttype]
enum DataKey {
    Admin = 1,
    Artists = 2,
    Listeners = 3,
    RoyaltiesState = 4,
    RoyaltiesPool = 5,
    Token = 6,
}

/// All errors this contract expects.
#[contracterror]
#[derive(Clone, Debug, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    AlreadyActive = 3,
    NotActive = 4,
    InsufficientFunds = 5,
    InvalidArtistAddress = 6,
    InvalidListenerAddress = 7,
    InvalidRoyaltiesPercentage = 8,
}

#[contract]
pub struct RoyaltiesContract;

#[contractimpl]
impl RoyaltiesContract {
    /// Initializes the contract with all the needed parameters.
    ///
    /// # Arguments
    ///
    /// - env - The environment for this contract.
    /// - admin - Admin account address.
    /// - token - The token contract address used for royalties payments.
    pub fn init(env: Env, admin: Address, token: Address) {
        admin.require_auth();
        let storage = env.storage().persistent();
        if storage
            .get::<_, RoyaltiesState>(&DataKey::RoyaltiesState)
            .is_some()
        {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }

        storage.set(&DataKey::Admin, &admin);
        storage.set(&DataKey::Token, &token);
        storage.set(&DataKey::RoyaltiesState, &RoyaltiesState::Initialized);
    }

    /// Adds an artist to the platform.
    ///
    /// # Arguments
    ///
    /// - `env` - The environment for this contract.
    /// - `artist` - Artist's account address.
    pub fn add_artist(env: Env, artist: Address) {
        let storage = env.storage().persistent();
        let admin = storage.get::<_, Address>(&DataKey::Admin).unwrap();
        admin.require_auth();

        let artists = storage.get::<_, Vec<Address>>(&DataKey::Artists).unwrap_or(vec![&env]);
        artists.push_back(artist);
        storage.set(&DataKey::Artists, &artists);
    }

    /// Adds a listener to the platform.
    ///
    /// # Arguments
    ///
    /// - `env` - The environment for this contract.
    /// - `listener` - Listener's account address.
    pub fn add_listener(env: Env, listener: Address) {
        let storage = env.storage().persistent();
        let admin = storage.get::<_, Address>(&DataKey::Admin).unwrap();
        admin.require_auth();

        let listeners = storage.get::<_, Vec<Address>>(&DataKey::Listeners).unwrap_or(vec![&env]);
        listeners.push_back(listener);
        storage.set(&DataKey::Listeners, &listeners);
    }

    /// Starts the royalties distribution.
    ///
    /// # Arguments
    ///
    /// - `env` - The environment for this contract.
    pub fn start_distribution(env: Env) {
        let storage = env.storage().persistent();
        let admin = storage.get::<_, Address>(&DataKey::Admin).unwrap();
        admin.require_auth();

        let royalties_state = storage
            .get::<_, RoyaltiesState>(&DataKey::RoyaltiesState)
            .unwrap();

        if royalties_state == RoyaltiesState::Active {
            panic_with_error!(&env, Error::AlreadyActive);
        }

        storage.set(&DataKey::RoyaltiesState, &RoyaltiesState::Active);
    }

    /// Stops the royalties distribution.
    ///
    /// # Arguments
    ///
    /// - `env` - The environment for this contract.
    pub fn stop_distribution(env: Env) {
        let storage = env.storage().persistent();
        let admin = storage.get::<_, Address>(&DataKey::Admin).unwrap();
        admin.require_auth();

        let royalties_state = storage
            .get::<_, RoyaltiesState>(&DataKey::RoyaltiesState)
            .unwrap();

        if royalties_state != RoyaltiesState::Active {
            panic_with_error!(&env, Error::NotActive);
        }

        storage.set(&DataKey::RoyaltiesState, &RoyaltiesState::Finished);
    }

    /// Distributes royalties to artists based on the royalties pool.
    ///
    /// # Arguments
    ///
    /// - `env` - The environment for this contract.
    pub fn distribute_royalties(env: Env) {
        let storage = env.storage().persistent();

        let royalties_state = storage
            .get::<_, RoyaltiesState>(&DataKey::RoyaltiesState)
            .unwrap();

        if royalties_state != RoyaltiesState::Active {
            panic_with_error!(&env, Error::NotActive);
        }

        let token = storage.get::<_, Address>(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token);

        let artists = storage.get::<_, Vec<Address>>(&DataKey::Artists).unwrap();
        let listeners = storage.get::<_, Vec<Address>>(&DataKey::Listeners).unwrap();
        let royalties_pool = storage.get::<_, i128>(&DataKey::RoyaltiesPool).unwrap_or(0);

        // Calculate royalties per artist
        let num_artists = artists.len() as i128;
        let royalties_per_artist = royalties_pool / num_artists;

        // Distribute royalties to artists
        for artist in artists.iter() {
            token_client.transfer(&env.current_contract_address(), artist, &royalties_per_artist);
        }

        // Clear royalties pool
        storage.set(&DataKey::RoyaltiesPool, &0);
    }

    /// Allows listeners to contribute to the royalties pool.
    ///
    /// # Arguments
    ///
    /// - `env` - The environment for this contract.
    /// - `amount` - The amount to contribute to the royalties pool.
    pub fn contribute(env: Env, amount: i128) {
        let storage = env.storage().persistent();
        let token = storage.get::<_, Address>(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token);

        token_client.transfer(&env.current_account(), &env.current_contract_address(), &amount);

        let royalties_pool = storage.get::<_, i128>(&DataKey::RoyaltiesPool).unwrap_or(0);
        storage.set(&DataKey::RoyaltiesPool, &(royalties_pool + amount));
    }
}
