use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use ic_cdk::storage;

use ic_cdk::export::Principal;

#[derive(Clone,CandidType, Deserialize)]
struct User{
    name:String,
    lastname:String,
    email:String,
    password:String,
    registrationYear:u16,

}

#[derive(CandidType, Deserialize)]
struct CreateUserArgs {
    name: String,
    lastname: String,
    email: String,
    password:String,
    registrationYear: u16,
}

#[derive(CandidType, Deserialize)]
struct Advert{
    title:String,
    description:String,
    price:u16,
    category:String,

}

#[derive(CandidType, Deserialize)]
enum category  {
    Icp,
    Web3,
    WebSite,
    Android,
    IOS,
    Windows,        
    SmartContract,
    Nft,
    Other,
}

#[derive(CandidType, Deserialize)]
enum ProgrammingLang{
    Motoko,
    Solidity,
    Vyper,
    Rust,
    JavaScript,
    TypeScript,
    Go,
    Python,
    Java,
    Csharp,
    CplusPlus,
}

#[derive(CandidType, Deserialize)]
enum userError {
    incorrectPassword,
    incorrectEmail, 
}
#[derive(CandidType, Deserialize)]
enum advertError {
    
    notitle,
    nodescription,
    noprice, 
    nocategory,
}



impl Storable for User {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

type Memory = VirtualMemory<DefaultMemoryImpl>;
const MAX_VALUE_SIZE: u32 = 10000;

impl BoundedStorable for Advert {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE; 
    const IS_FIXED_SIZE: bool = false;
}
impl BoundedStorable for User {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE; 
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static EVENTS_MAP: RefCell<StableBTreeMap<u64, User, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))), 
        )
    );
}

#[ic_cdk::update]
fn create_user(args: CreateUserArgs) {
    let new_user = User {
        name: args.name,
        lastname: args.lastname,
        email: args.email,
        password:args.password,
        registrationYear: args.registrationYear,
    };
    let key = (ic_cdk::caller());
    storage::stable_save((key, new_user)).unwrap();
}

#[ic_cdk_macros::query]
fn get_user(email: String) -> Option<User> {
    let key = (ic_cdk::caller(), email);
    match ic_cdk::storage::stable_restore::<(ic_cdk::Principal, String), User>(&key) {
        Ok(Some(user)) => Some(user),
        _ => None,
    }
}
#[ic_cdk_macros::query]
fn get_all_users() -> Vec<User> {
    EVENTS_MAP.with(|events_map_ref| {
        let events_map = &*events_map_ref.borrow();
        events_map
            .values()
            .map(|event| event.clone())
            .collect()
    })
}
#[ic_cdk_macros::query]
fn get_users_sorted_by_name() -> Vec<User> {
    let mut users = get_all_users();
    users.sort_by(|a, b| a.name.cmp(&b.name));
    users
}

#[ic_cdk_macros::query]
fn list_users_by_email(email: String) -> Vec<User> {
    let mut users = vec![];
    EVENTS_MAP.with(|events_map_ref| {
        let events_map = events_map_ref.borrow();
        for (_, user) in events_map.iter() {
            if user.email == email {
                users.push(user.clone());
            }
        }
    });
    users
}
#[ic_cdk::update]
fn create_advert(advert: Advert) {
    let key = (ic_cdk::caller(), &advert.title);
    storage::stable_save((key, advert)).unwrap_or_else(|_| ic_cdk::trap("Error saving advert"));
}
#[ic_cdk::update]
fn publish_advert(title: String) {
    let key = (ic_cdk::caller(), &title);
    match storage::stable_get::<(ic_cdk::Principal, String), Advert>(key) {
        Ok(Some(advert)) => {
          
            println!("Published advert: {:?}", advert);
        },
        _ => ic_cdk::trap("Advert not found"), 
    }
}

#[ic_cdk_macros::query]
fn list_adverts_by_category(category: String) -> Vec<Advert> {
    let mut adverts = vec![];
    EVENTS_MAP.with(|events_map_ref| {
        let events_map = events_map_ref.borrow();
        for (_, advert) in events_map.iter() {
            if advert.category == category {
                adverts.push(advert.clone());
            }
        }
    });
    adverts
}

#[ic_cdk_macros::query]
fn check_credentials(username: String, password: String) -> Option<userError> {
    let key = (ic_cdk::caller(), &username);
    match storage::stable_get::<(ic_cdk::Principal, String), User>(key) {
        Ok(Some(user)) => {
            if user.password == password {
                None 
            } else {
                Some(userError::incorrectPassword) 
            }
        },
        _ => Some(userError::incorrectEmail), 
    }
}

#[ic_cdk::update]
fn publish_advert1(advert: Advert) -> Result<(), advertError> {
    if advert.title.is_empty() {
        return Err(advertError::notitle);
    }
    if advert.description.is_empty() {
        return Err(advertError::nodescription);
    }
    if advert.price == 0 {
        return Err(advertError::noprice);
    }
    if advert.category.is_empty() {
        return Err(advertError::nocategory);
    }

   
    let key = (ic_cdk::caller(), &advert.title);
    storage::stable_save((key, advert)).unwrap_or_else(|_| ic_cdk::trap("Error saving advert"));

    Ok(())
}

