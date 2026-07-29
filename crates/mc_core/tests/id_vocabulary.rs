use mc_core::ids::{
    AbilityId, CharId, EnemyId, FlagId, ItemId, PoisonId, RegionId, SceneId, TechId,
};

macro_rules! assert_vocabulary {
    ($type:ty, $count:expr, [$($name:expr),+ $(,)?]) => {{
        let expected = [$($name),+];
        assert_eq!(expected.len(), $count);
        for (raw, name) in expected.into_iter().enumerate() {
            let id = <$type>::from_raw(raw as u16);
            assert_eq!(id.raw(), raw as u16);
            assert_eq!(id.name(), name);
        }
        assert_eq!(<$type>::from_raw($count as u16).name(), "UNKNOWN");
    }};
}

#[test]
fn every_locked_identifier_has_the_specified_display_name() {
    assert_vocabulary!(
        RegionId,
        RegionId::COUNT,
        [
            "Marseille",
            "Château d'If",
            "Monte Cristo",
            "Rome",
            "Paris Faubourg",
            "Paris Salon",
            "Normandy",
            "Lyon",
            "Strasbourg",
            "Méditerranée",
            "Orient",
            "Greece",
            "Albania",
            "Morcerf Estate",
            "Villefort Mansion",
        ]
    );
    assert_vocabulary!(
        EnemyId,
        EnemyId::COUNT,
        [
            "Bandit",
            "Soldier",
            "Assassin",
            "Smuggler",
            "Gendarme",
            "Corsican",
            "Cretan",
            "Albanian",
            "Greek Rebel",
            "Ottoman",
            "Spy",
            "Bodyguard",
            "Jailer",
            "Guard",
            "Agent",
        ]
    );
    assert_vocabulary!(
        CharId,
        CharId::COUNT,
        [
            "Edmond",
            "Abbé Faria",
            "Haydée",
            "Mercedes",
            "Albert",
            "Fernand",
            "Danglars",
            "Villefort",
            "Valentine",
            "Noirtier",
            "Bertuccio",
            "Heloise de Villefort",
        ]
    );
    assert_vocabulary!(
        AbilityId,
        AbilityId::COUNT,
        [
            "Lunge",
            "Riposte",
            "Feint",
            "Flash Powder",
            "Antidote",
            "Stimulant",
            "Analyze",
            "Weather Eye",
            "Ballistic Calc",
            "Probability Shift",
            "Persuade",
            "Decipher",
            "Inspire",
            "Tactics",
            "Bribe",
            "Fund",
        ]
    );
    assert_vocabulary!(
        ItemId,
        ItemId::COUNT,
        [
            "Potion",
            "Hi-Potion",
            "Antidote",
            "Panacea",
            "Smoke Bomb",
            "Phial of Brucine",
            "Treasure Map",
            "Edouard's Locket",
        ]
    );
    assert_vocabulary!(
        SceneId,
        SceneId::COUNT,
        [
            "Arrest",
            "Faria Meeting",
            "Treasure Reveal",
            "Escape",
            "Sinbad",
            "Roman Carnival",
            "Morcerf Reveal",
            "Danglars' Ruin",
            "Villefort's Justice",
            "Heloise's Poison",
            "Valentine Saved",
            "Mercedes' Goodbye",
            "Edouard",
            "Final Confrontation",
        ]
    );
    assert_vocabulary!(
        FlagId,
        FlagId::COUNT,
        [
            "Arrested",
            "Faria Met",
            "Treasure Known",
            "Escaped",
            "Comte Identity",
            "Sinbad Visited",
            "Morcerf Dossier",
            "Morcerf Yanina Dossier",
            "Albert Withdrawn",
            "Danglars Letter",
            "Villefort Dossier",
            "Heloise Poisoning",
            "Valentine Safe",
            "Mercedes Recognition",
            "Edouard Truth",
            "Fernand Confronted",
            "Danglars Confronted",
            "Villefort Confronted",
            "Mercedes Forgiven",
            "Final Phase 1",
            "Final Phase 2",
            "Final Phase 3",
        ]
    );
    assert_vocabulary!(
        PoisonId,
        PoisonId::COUNT,
        [
            "Brucine",
            "Aconite",
            "Belladonna",
            "Arsenic",
            "Hydrocyanic Acid",
        ]
    );
    assert_vocabulary!(
        TechId,
        TechId::COUNT,
        [
            "Attack",
            "Lunge",
            "Riposte",
            "Feint",
            "Flash Powder",
            "Antidote",
            "Stimulant",
            "Analyze",
            "Ballistic Calc",
            "Probability Shift",
            "Persuade",
            "Mass Persuade",
            "Inspire",
            "Tactics",
            "Bribe",
            "Fund",
        ]
    );
}
