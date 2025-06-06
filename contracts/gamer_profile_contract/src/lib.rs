#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod gamer_profile_contract {
    use ink::storage::Mapping;
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;

    #[derive(scale::Encode, scale::Decode, Clone, Debug, Default, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct GamerProfile {
        username: Option<String>,
        avatar_uri: Option<String>,
        achievements: Vec<String>,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct GamerProfileContract {
        profiles: Mapping<AccountId, GamerProfile>,
    }

    impl GamerProfileContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        #[ink(message)]
        pub fn create_profile(&mut self, username: String, avatar_uri: String) {
            let caller = self.env().caller();
            let profile = GamerProfile {
                username: Some(username),
                avatar_uri: Some(avatar_uri),
                achievements: Vec::new(),
            };
            self.profiles.insert(caller, &profile);
        }

        #[ink(message)]
        pub fn add_achievement(&mut self, achievement: String) {
            let caller = self.env().caller();
            if let Some(mut profile) = self.profiles.get(caller) {
                profile.achievements.push(achievement);
                self.profiles.insert(caller, &profile);
            }
            // TODO: Consider adding an error event if the profile doesn't exist
        }

        #[ink(message)]
        pub fn get_profile(&self, account: AccountId) -> Option<GamerProfile> {
            self.profiles.get(account)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        fn default_accounts() -> test::DefaultAccounts<ink::env::DefaultEnvironment> {
            test::default_accounts::<Environment>()
        }

        fn set_caller(sender: AccountId) {
            test::set_caller::<Environment>(sender);
        }

        #[ink::test]
        fn new_works() {
            let contract = GamerProfileContract::new();
            let accounts = default_accounts();
            assert_eq!(contract.get_profile(accounts.alice), None);
        }

        #[ink::test]
        fn create_profile_works() {
            let mut contract = GamerProfileContract::new();
            let accounts = default_accounts();
            set_caller(accounts.alice);

            contract.create_profile("Alice".into(), "alice_avatar.png".into());
            let profile = contract.get_profile(accounts.alice).expect("Profile should exist");
            assert_eq!(profile.username, Some("Alice".into()));
            assert_eq!(profile.avatar_uri, Some("alice_avatar.png".into()));
            assert_eq!(profile.achievements.len(), 0);
        }

        #[ink::test]
        fn add_achievement_works() {
            let mut contract = GamerProfileContract::new();
            let accounts = default_accounts();
            set_caller(accounts.alice);

            contract.create_profile("Alice".into(), "alice_avatar.png".into());
            contract.add_achievement("First Quest Completed".into());

            let profile = contract.get_profile(accounts.alice).expect("Profile should exist");
            assert_eq!(profile.achievements.len(), 1);
            assert_eq!(profile.achievements[0], "First Quest Completed".into());
        }

        #[ink::test]
        fn add_achievement_no_profile() {
            let mut contract = GamerProfileContract::new();
            let accounts = default_accounts();
            set_caller(accounts.bob); // Bob has no profile

            contract.add_achievement("Should not be added".into());
            // We expect nothing to happen, no panic, and Bob still has no profile
            // or his profile remains unchanged if he had one (which he doesn't here).
            // The current implementation doesn't error out, so we just check profile remains None.
            assert_eq!(contract.get_profile(accounts.bob), None);
        }

        #[ink::test]
        fn get_profile_non_existent() {
            let contract = GamerProfileContract::new();
            let accounts = default_accounts();
            assert_eq!(contract.get_profile(accounts.bob), None);
        }
    }
}
