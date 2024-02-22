use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use ic_cdk::storage;
use std::collections::BTreeMap;
use ic_cdk::export::Principal;


#[derive(Clone,CandidType, Deserialize)]
struct User{
    name:String,
    lastname:String,
    email:String,
    password:String,
    registrationYear:u32,

}

#[derive(CandidType, Deserialize)]
struct CreateUserArgs {
    name: String,
    lastname: String,
    email: String,
    password:String,
    registrationYear: u32,
}

#[derive(CandidType, Deserialize,Clone)]
struct Advert{
    title:String,
    description:String,
    price:String,
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
    unfilled(String),
    incorrectEmail(String),
    incorrectPassword(String),
}
#[derive(CandidType,Deserialize)]
enum userResult {
    Success(String),
    Error(userError),
    
}



impl Storable for User {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}
impl Storable for Advert {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}
type Memory = VirtualMemory<DefaultMemoryImpl>;
const MAX_VALUE_SIZE: u32 = 100;
impl BoundedStorable for User {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE; 
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Advert {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE; 
    const IS_FIXED_SIZE: bool = false;
}
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static USER_MAP: RefCell<StableBTreeMap<u64, User, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|p| p.borrow().get(MemoryId::new(2))), 
        )
    );
    static Advert_Map: RefCell<StableBTreeMap<u64, Advert, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|p| p.borrow().get(MemoryId::new(2))), 
        )
    );
}

#[ic_cdk::update]
fn create_user(name:String,lastname:String,email:String,password: String,registrationYear:u32,)->Result<userResult, userError> {
    if name.is_empty() || lastname.is_empty() || email.is_empty()||password.is_empty(){
        return Err(userError::unfilled("boş alanlar mevcut".to_string()));
    }
    
   USER_MAP.with(|p|{
    let mut user_map = p.borrow_mut();
    let new_user = User{
        name:name,
        lastname:lastname,
        password:password,
        email:email,
        registrationYear:registrationYear, 
    };
    let new_user_id=user_map.len();
user_map.insert(new_user_id,new_user);
   });
   Ok(userResult::Success("üyeliğiniz oluşturulmuştur".to_string()))
}

#[ic_cdk_macros::query]
fn sort_users()->Vec<User> {
    let mut users = Vec::new();
    USER_MAP.with(|p|{
        let user_map=p.borrow();
        let mut iter = user_map.iter();
    while let Some((_, user)) = iter.next() {
    users.push(user.clone());
}
});
users    
} 
#[ic_cdk::update]
fn create_advert(title: String, description: String, price: String, category: String)->Result<userResult, userError> {
    if title.is_empty() || description.is_empty() || price.is_empty()||category.is_empty(){
        return Err(userError::unfilled("boş alanlar mevcut".to_string()));
    }
    
   Advert_Map.with(|p|{
    let mut advert_map = p.borrow_mut();
    let new_advert = Advert{
        title:title,
        description:description,
        price:price,
        category:category,
        
    };
   });
   Ok(userResult::Success("ilanınız oluşturulmuştur".to_string()))
}
#[ic_cdk_macros::query]
fn sort_adverts()->Vec<Advert> {
    let mut adverts = Vec::new();
    Advert_Map.with(|p|{
        let advert_map=p.borrow();
       
    for (_, advert) in advert_map.iter() {
    adverts.push(advert.clone());
}
});
adverts    
} 

#[ic_cdk_macros::query]
fn get_user_by_email(email: String) -> Option<User> {
    USER_MAP.with(|user_map_ref| {
        let user_map = user_map_ref.borrow();
        for (_, user) in user_map.iter() {
            if user.email == email {
                return Some(user.clone());
            }
        }
        None
    })
}

#[ic_cdk::update]
fn delete_user_by_email(email: String) -> Result<userResult, userError> {
    USER_MAP.with(|user_map| {
        let mut to_remove = None;
        for (id, user) in user_map.borrow().iter() {
            if user.email == email {
                to_remove = Some(id.clone());
                break;
            }
        }
        if let Some(id) = to_remove {
            user_map.borrow_mut().remove(&id);
            Ok(userResult::Success("Kullanıcı silindi.".to_string()))
        } else {
            Err(userError::unfilled("Kullanıcı bulunamadı.".to_string()))
        }
    })
}


#[ic_cdk_macros::query]
fn list_adverts_by_category() -> Vec<String> {
    let mut adverts = vec![];
    Advert_Map.with(|advert_map_ref| {
        let advert_map = advert_map_ref.borrow();
        for (_, advert) in advert_map.iter() {
            
            adverts.push(advert.category.clone());            
        }
    });
    adverts
} 
#[ic_cdk::update]
fn login_user(email: String, password: String) -> Result<userResult, userError> {
    USER_MAP.with(|user_map_ref| {
        let user_map = user_map_ref.borrow();
        for (_, user) in user_map.iter() {
            if user.email == email {
                if user.password == password {
                    return Ok(userResult::Success("Giriş başarılı.".to_string()));
                } else {
                    return Err(userError::incorrectPassword("Şifre yanlış.".to_string()));
                }
            }
        }
        Err(userError::incorrectEmail("E-posta adresi bulunamadı.".to_string()))
    })
}



