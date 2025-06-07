#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod gamer_profile_contract {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct GamerProfile {
        name: String,
        level: u32,
        experience: u64,
    }

    #[ink(storage)]
    pub struct GamerProfileContract {
        profiles: Mapping<AccountId, GamerProfile>,
        achievements: Mapping<(AccountId, u32), String>,
        achievement_counts: Mapping<AccountId, u32>,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        ProfileAlreadyExists,
        ProfileNotFound,
        TooManyAchievements,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl GamerProfileContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                profiles: Mapping::new(),
                achievements: Mapping::new(),
                achievement_counts: Mapping::new(),
            }
        }

        #[ink(message)]
        pub fn create_profile(&mut self, name: String) -> Result<()> {
            let caller = self.env().caller();
            
            if self.profiles.contains(caller) {
                return Err(Error::ProfileAlreadyExists);
            }

            let profile = GamerProfile {
                name,
                level: 1,
                experience: 0,
            };

            self.profiles.insert(caller, &profile);
            self.achievement_counts.insert(caller, &0);
            Ok(())
        }

        #[ink(message)]
        pub fn get_profile(&self, account: AccountId) -> Option<GamerProfile> {
            self.profiles.get(account)
        }
        
        #[ink(message)]
        pub fn get_achievements(&self, account: AccountId) -> Vec<String> {
            let count = self.achievement_counts.get(account).unwrap_or(0);
            let mut achievements = Vec::with_capacity(count as usize);
            for i in 0..count {
                if let Some(achievement) = self.achievements.get((account, i)) {
                    achievements.push(achievement);
                }
            }
            achievements
        }

        #[ink(message)]
        pub fn add_achievement(&mut self, achievement: String) -> Result<()> {
            let caller = self.env().caller();
            
            // Verify profile exists
            if !self.profiles.contains(caller) {
                return Err(Error::ProfileNotFound);
            }
            
            // Get current achievement count and increment
            let count = self.achievement_counts.get(caller).unwrap_or(0);
            let new_count = count.checked_add(1).ok_or(Error::TooManyAchievements)?;
            
            // Store the new achievement
            self.achievements.insert((caller, count), &achievement);
            self.achievement_counts.insert(caller, &new_count);
            
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn create_profile_works() {
            let mut contract = GamerProfileContract::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            
            // Test creating a profile
            assert_eq!(
                contract.create_profile("Alice".to_string()),
                Ok(())
            );
            
            // Test getting the profile
            let profile = contract.get_profile(accounts.alice).unwrap();
            assert_eq!(profile.name, "Alice");
            assert_eq!(profile.level, 1);
            
            // Test duplicate profile creation fails
            assert_eq!(
                contract.create_profile("Alice2".to_string()),
                Err(Error::ProfileAlreadyExists)
            );
        }

        #[ink::test]
        fn add_achievement_works() {
            let mut contract = GamerProfileContract::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            
            // Create a profile first
            contract.create_profile("Bob".to_string()).unwrap();
            
            // Add an achievement
            assert_eq!(
                contract.add_achievement("First Blood".to_string()),
                Ok(())
            );
            
            // Verify the achievement was added
            let profile = contract.get_profile(accounts.alice).unwrap();
            assert_eq!(profile.achievements, vec!["First Blood".to_string()]);
        }
    }
}
