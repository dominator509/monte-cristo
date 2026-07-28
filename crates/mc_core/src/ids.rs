//! Locked identifier types for the game's built-in vocabulary.
//!
//! Each is a compact newtype backed by u16, with a `&'static str` table for
//! display and debugging. Identifiers come from SPEC-009 and SPEC-010.

use serde::{Deserialize, Serialize};

/// A compact region identifier backed by u16.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct RegionId(u16);

impl RegionId {
    pub const COUNT: usize = 15;
    pub const R01_MARSEILLE: Self = Self(0);
    pub const R02_CHATEAU_DIF: Self = Self(1);
    pub const R03_MONTE_CRISTO: Self = Self(2);
    pub const R04_ROME: Self = Self(3);
    pub const R05_PARIS_FAUBOURG: Self = Self(4);
    pub const R06_PARIS_SALON: Self = Self(5);
    pub const R07_NORMANDY: Self = Self(6);
    pub const R08_LYON: Self = Self(7);
    pub const R09_STRASBOURG: Self = Self(8);
    pub const R10_MEDITERRANEE: Self = Self(9);
    pub const R11_ORIENT: Self = Self(10);
    pub const R12_GREECE: Self = Self(11);
    pub const R13_ALBANIA: Self = Self(12);
    pub const R14_MORCERF_ESTATE: Self = Self(13);
    pub const R15_VILLEFORT_MANSION: Self = Self(14);

    pub const fn from_raw(v: u16) -> Self {
        Self(v)
    }
    pub const fn raw(self) -> u16 {
        self.0
    }

    pub fn name(self) -> &'static str {
        match self.0 {
            0 => "Marseille",
            1 => "Château d'If",
            2 => "Monte Cristo",
            3 => "Rome",
            4 => "Paris Faubourg",
            5 => "Paris Salon",
            6 => "Normandy",
            7 => "Lyon",
            8 => "Strasbourg",
            9 => "Méditerranée",
            10 => "Orient",
            11 => "Greece",
            12 => "Albania",
            13 => "Morcerf Estate",
            14 => "Villefort Mansion",
            _ => "UNKNOWN",
        }
    }
}

/// A compact enemy identifier backed by u16.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct EnemyId(u16);

impl EnemyId {
    pub const COUNT: usize = 15;
    pub const ENM_BANDIT: Self = Self(0);
    pub const ENM_SOLDIER: Self = Self(1);
    pub const ENM_ASSASSIN: Self = Self(2);
    pub const ENM_SMUGGLER: Self = Self(3);
    pub const ENM_GENDARME: Self = Self(4);
    pub const ENM_CORSICAN: Self = Self(5);
    pub const ENM_CRETAN: Self = Self(6);
    pub const ENM_ALBANIAN: Self = Self(7);
    pub const ENM_GREEK_REBEL: Self = Self(8);
    pub const ENM_OTTOMAN: Self = Self(9);
    pub const ENM_SPY: Self = Self(10);
    pub const ENM_BODYGUARD: Self = Self(11);
    pub const ENM_JAILER: Self = Self(12);
    pub const ENM_GUARD: Self = Self(13);
    pub const ENM_AGENT: Self = Self(14);

    pub const fn from_raw(v: u16) -> Self {
        Self(v)
    }
    pub const fn raw(self) -> u16 {
        self.0
    }

    pub fn name(self) -> &'static str {
        match self.0 {
            0 => "Bandit",
            1 => "Soldier",
            2 => "Assassin",
            3 => "Smuggler",
            4 => "Gendarme",
            5 => "Corsican",
            6 => "Cretan",
            7 => "Albanian",
            8 => "Greek Rebel",
            9 => "Ottoman",
            10 => "Spy",
            11 => "Bodyguard",
            12 => "Jailer",
            13 => "Guard",
            14 => "Agent",
            _ => "UNKNOWN",
        }
    }
}

/// A compact character identifier backed by u16.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct CharId(u16);

impl CharId {
    pub const COUNT: usize = 12;
    pub const CHR_EDMOND: Self = Self(0);
    pub const CHR_ABBE_FARIA: Self = Self(1);
    pub const CHR_HAYDEE: Self = Self(2);
    pub const CHR_MERCEDES: Self = Self(3);
    pub const CHR_ALBERT: Self = Self(4);
    pub const CHR_FERNAND: Self = Self(5);
    pub const CHR_DANGLARS: Self = Self(6);
    pub const CHR_VILLEFORT: Self = Self(7);
    pub const CHR_VALENTINE: Self = Self(8);
    pub const CHR_NOIRTIER: Self = Self(9);
    pub const CHR_BERTUCCIO: Self = Self(10);
    pub const CHR_HELOISE: Self = Self(11);

    pub const fn from_raw(v: u16) -> Self {
        Self(v)
    }
    pub const fn raw(self) -> u16 {
        self.0
    }

    pub fn name(self) -> &'static str {
        match self.0 {
            0 => "Edmond",
            1 => "Abbé Faria",
            2 => "Haydée",
            3 => "Mercedes",
            4 => "Albert",
            5 => "Fernand",
            6 => "Danglars",
            7 => "Villefort",
            8 => "Valentine",
            9 => "Noirtier",
            10 => "Bertuccio",
            11 => "Heloise de Villefort",
            _ => "UNKNOWN",
        }
    }
}

/// A compact ability identifier.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct AbilityId(u16);

impl AbilityId {
    pub const COUNT: usize = 16;
    pub const ABL_FENCING_LUNGE: Self = Self(0);
    pub const ABL_FENCING_RIPOSTE: Self = Self(1);
    pub const ABL_FENCING_FEINT: Self = Self(2);
    pub const ABL_CHEM_FLASH: Self = Self(3);
    pub const ABL_CHEM_ANTIDOTE: Self = Self(4);
    pub const ABL_CHEM_STIM: Self = Self(5);
    pub const ABL_NATPHIL_ANALYZE: Self = Self(6);
    pub const ABL_NATPHIL_WEATHER: Self = Self(7);
    pub const ABL_MATH_BALLISTIC: Self = Self(8);
    pub const ABL_MATH_PROBABILITY: Self = Self(9);
    pub const ABL_LANG_PERSUADE: Self = Self(10);
    pub const ABL_LANG_DECIPHER: Self = Self(11);
    pub const ABL_HIST_INSPIRE: Self = Self(12);
    pub const ABL_HIST_TACTICS: Self = Self(13);
    pub const ABL_ECON_BRIBE: Self = Self(14);
    pub const ABL_ECON_FUND: Self = Self(15);

    pub const fn from_raw(v: u16) -> Self {
        Self(v)
    }
    pub const fn raw(self) -> u16 {
        self.0
    }

    pub fn name(self) -> &'static str {
        match self.0 {
            0 => "Lunge",
            1 => "Riposte",
            2 => "Feint",
            3 => "Flash Powder",
            4 => "Antidote",
            5 => "Stimulant",
            6 => "Analyze",
            7 => "Weather Eye",
            8 => "Ballistic Calc",
            9 => "Probability Shift",
            10 => "Persuade",
            11 => "Decipher",
            12 => "Inspire",
            13 => "Tactics",
            14 => "Bribe",
            15 => "Fund",
            _ => "UNKNOWN",
        }
    }
}

/// A compact item identifier.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ItemId(u16);

impl ItemId {
    pub const COUNT: usize = 8;
    pub const ITM_POTION: Self = Self(0);
    pub const ITM_HI_POTION: Self = Self(1);
    pub const ITM_ANTIDOTE: Self = Self(2);
    pub const ITM_PANACEA: Self = Self(3);
    pub const ITM_SMOKE_BOMB: Self = Self(4);
    pub const ITM_PHIAL_BRUCINE: Self = Self(5);
    pub const ITM_TREASURE_MAP: Self = Self(6);
    pub const ITM_EDOUARD_LOCKET: Self = Self(7);

    pub const fn from_raw(v: u16) -> Self {
        Self(v)
    }
    pub const fn raw(self) -> u16 {
        self.0
    }

    pub fn name(self) -> &'static str {
        match self.0 {
            0 => "Potion",
            1 => "Hi-Potion",
            2 => "Antidote",
            3 => "Panacea",
            4 => "Smoke Bomb",
            5 => "Phial of Brucine",
            6 => "Treasure Map",
            7 => "Edouard's Locket",
            _ => "UNKNOWN",
        }
    }
}

/// A compact scene identifier.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct SceneId(u16);

impl SceneId {
    pub const COUNT: usize = 14;
    pub const SCN_ARREST: Self = Self(0);
    pub const SCN_FARIA_MEETING: Self = Self(1);
    pub const SCN_TREASURE_REVEAL: Self = Self(2);
    pub const SCN_ESCAPE: Self = Self(3);
    pub const SCN_SINDBAD: Self = Self(4);
    pub const SCN_ROMAN_CARNIVAL: Self = Self(5);
    pub const SCN_MORCERF_REVEAL: Self = Self(6);
    pub const SCN_DANGLARS_RUIN: Self = Self(7);
    pub const SCN_VILLEFORT_JUSTICE: Self = Self(8);
    pub const SCN_HELOISE_POISON: Self = Self(9);
    pub const SCN_VALENTINE_SAVED: Self = Self(10);
    pub const SCN_MERCEDES_GOODBYE: Self = Self(11);
    pub const SCN_EDOUARD: Self = Self(12);
    pub const SCN_FINAL_CONFRONTATION: Self = Self(13);

    pub const fn from_raw(v: u16) -> Self {
        Self(v)
    }
    pub const fn raw(self) -> u16 {
        self.0
    }

    pub fn name(self) -> &'static str {
        match self.0 {
            0 => "Arrest",
            1 => "Faria Meeting",
            2 => "Treasure Reveal",
            3 => "Escape",
            4 => "Sinbad",
            5 => "Roman Carnival",
            6 => "Morcerf Reveal",
            7 => "Danglars' Ruin",
            8 => "Villefort's Justice",
            9 => "Heloise's Poison",
            10 => "Valentine Saved",
            11 => "Mercedes' Goodbye",
            12 => "Edouard",
            13 => "Final Confrontation",
            _ => "UNKNOWN",
        }
    }
}

/// A compact flag identifier.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct FlagId(u16);

impl FlagId {
    pub const COUNT: usize = 22;
    pub const FLG_ARRESTED: Self = Self(0);
    pub const FLG_FARIA_MET: Self = Self(1);
    pub const FLG_TREASURE_KNOWN: Self = Self(2);
    pub const FLG_ESCAPED: Self = Self(3);
    pub const FLG_COMTE_IDENTITY: Self = Self(4);
    pub const FLG_SINDBAD_VISITED: Self = Self(5);
    pub const FLG_MORCERF_DOSSIER: Self = Self(6);
    pub const FLG_MORCERF_YANINA_DOSSIER: Self = Self(7);
    pub const FLG_MORCERF_ALBERT_WITHDRAWN: Self = Self(8);
    pub const FLG_DANGLARS_LETTER: Self = Self(9);
    pub const FLG_VILLEFORT_DOSSIER: Self = Self(10);
    pub const FLG_HELOISE_POISONING: Self = Self(11);
    pub const FLG_VALENTINE_SAFE: Self = Self(12);
    pub const FLG_MERCEDES_RECOGNITION: Self = Self(13);
    pub const FLG_EDOUARD_TRUTH: Self = Self(14);
    pub const FLG_FERNAND_CONFRONTED: Self = Self(15);
    pub const FLG_DANGLARS_CONFRONTED: Self = Self(16);
    pub const FLG_VILLEFORT_CONFRONTED: Self = Self(17);
    pub const FLG_MERCEDES_FORGIVEN: Self = Self(18);
    pub const FLG_FINAL_PHASE1: Self = Self(19);
    pub const FLG_FINAL_PHASE2: Self = Self(20);
    pub const FLG_FINAL_PHASE3: Self = Self(21);

    pub const fn from_raw(v: u16) -> Self {
        Self(v)
    }
    pub const fn raw(self) -> u16 {
        self.0
    }

    pub fn name(self) -> &'static str {
        match self.0 {
            0 => "Arrested",
            1 => "Faria Met",
            2 => "Treasure Known",
            3 => "Escaped",
            4 => "Comte Identity",
            5 => "Sinbad Visited",
            6 => "Morcerf Dossier",
            7 => "Morcerf Yanina Dossier",
            8 => "Albert Withdrawn",
            9 => "Danglars Letter",
            10 => "Villefort Dossier",
            11 => "Heloise Poisoning",
            12 => "Valentine Safe",
            13 => "Mercedes Recognition",
            14 => "Edouard Truth",
            15 => "Fernand Confronted",
            16 => "Danglars Confronted",
            17 => "Villefort Confronted",
            18 => "Mercedes Forgiven",
            19 => "Final Phase 1",
            20 => "Final Phase 2",
            21 => "Final Phase 3",
            _ => "UNKNOWN",
        }
    }
}

/// A compact poison identifier.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct PoisonId(u16);

impl PoisonId {
    pub const COUNT: usize = 5;
    pub const PSN_BRUCINE: Self = Self(0);
    pub const PSN_ACONITE: Self = Self(1);
    pub const PSN_BELLADONNA: Self = Self(2);
    pub const PSN_ARSENIC: Self = Self(3);
    pub const PSN_HYDROCYANIC: Self = Self(4);

    pub const fn from_raw(v: u16) -> Self {
        Self(v)
    }
    pub const fn raw(self) -> u16 {
        self.0
    }

    pub fn name(self) -> &'static str {
        match self.0 {
            0 => "Brucine",
            1 => "Aconite",
            2 => "Belladonna",
            3 => "Arsenic",
            4 => "Hydrocyanic Acid",
            _ => "UNKNOWN",
        }
    }
}

/// A compact technique identifier.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct TechId(u16);

impl TechId {
    pub const COUNT: usize = 16;
    pub const TEC_ATTACK: Self = Self(0);
    pub const TEC_LUNGE: Self = Self(1);
    pub const TEC_RIPOSTE: Self = Self(2);
    pub const TEC_FEINT: Self = Self(3);
    pub const TEC_FLASH_POWDER: Self = Self(4);
    pub const TEC_ANTIDOTE: Self = Self(5);
    pub const TEC_STIMULANT: Self = Self(6);
    pub const TEC_ANALYZE: Self = Self(7);
    pub const TEC_BALLISTIC: Self = Self(8);
    pub const TEC_PROBABILITY: Self = Self(9);
    pub const TEC_PERSUADE: Self = Self(10);
    pub const TEC_PERSUADE_ALL: Self = Self(11);
    pub const TEC_INSPIRE: Self = Self(12);
    pub const TEC_TACTICS: Self = Self(13);
    pub const TEC_BRIBE: Self = Self(14);
    pub const TEC_FUND: Self = Self(15);

    pub const fn from_raw(v: u16) -> Self {
        Self(v)
    }
    pub const fn raw(self) -> u16 {
        self.0
    }

    pub fn name(self) -> &'static str {
        match self.0 {
            0 => "Attack",
            1 => "Lunge",
            2 => "Riposte",
            3 => "Feint",
            4 => "Flash Powder",
            5 => "Antidote",
            6 => "Stimulant",
            7 => "Analyze",
            8 => "Ballistic Calc",
            9 => "Probability Shift",
            10 => "Persuade",
            11 => "Mass Persuade",
            12 => "Inspire",
            13 => "Tactics",
            14 => "Bribe",
            15 => "Fund",
            _ => "UNKNOWN",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_count() {
        assert_eq!(RegionId::COUNT, 15);
    }

    #[test]
    fn region_names() {
        assert_eq!(RegionId::R01_MARSEILLE.name(), "Marseille");
        assert_eq!(RegionId::R15_VILLEFORT_MANSION.name(), "Villefort Mansion");
    }

    #[test]
    fn ordered_comparison() {
        assert!(RegionId::R01_MARSEILLE < RegionId::R02_CHATEAU_DIF);
        assert!(EnemyId::ENM_BANDIT == EnemyId::ENM_BANDIT);
    }

    #[test]
    fn round_trip() {
        let id = RegionId::R10_MEDITERRANEE;
        let raw = id.raw();
        assert_eq!(RegionId::from_raw(raw), id);
    }

    #[test]
    fn unknown_returns_default_name() {
        assert_eq!(RegionId::from_raw(255).name(), "UNKNOWN");
    }

    #[test]
    fn enemy_count() {
        assert_eq!(EnemyId::COUNT, 15);
    }

    #[test]
    fn char_count() {
        assert_eq!(CharId::COUNT, 12);
    }

    #[test]
    fn flag_count() {
        assert_eq!(FlagId::COUNT, 22);
    }
}
