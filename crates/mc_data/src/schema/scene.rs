//! Content schema types for scene data.
//!
//! These are the static, authored scene definitions used by the content pipeline.
//! Runtime execution types live in mc_core::scene.

use mc_core::flags::FlagExpr;
use serde::{Deserialize, Serialize};

/// Act of the overarching story.
// Names use ROMAN_NUMERAL format per SPEC-002 section 5 scene schema.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Act {
    /// Edmond Dantès is arrested.
    ActI_ARREST,
    /// Edmond is in the Château d'If.
    ActII_CHATEAU,
    /// Edmond finds the treasure of Monte Cristo.
    ActIII_TREASURE,
    /// Edmond tours Europe as the Count.
    ActIV_TOUR,
    /// Edmond exacts vengeance in Paris.
    ActV_PARIS,
    /// Justice is served.
    ActVI_JUSTICE,
    /// The final act.
    ActVII_FINAL,
}

/// A trust effect: which character is affected and by how much.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustEffect(pub String, pub i32);

/// Side effects triggered by entering or leaving a scene node.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effects {
    /// Flags to set when this effect is applied.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub set_flags: Vec<String>,
    /// Flags to clear when this effect is applied.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clear_flags: Vec<String>,
    /// Items to consume (remove from inventory).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consume: Vec<String>,
    /// Items to grant (add to inventory).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grant: Vec<String>,
    /// Trust adjustments for characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust: Option<Vec<TrustEffect>>,
    /// Mask meter adjustment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: Option<i32>,
}

/// A single player choice within a scene node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    /// Localisation key for the choice text.
    pub text_key: String,
    /// ID of the destination node.
    pub to: String,
    /// Optional trust effects triggered by this choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust: Option<Vec<TrustEffect>>,
    /// Optional flag condition required for this choice to appear.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<FlagExpr>,
}

/// A single node within a scene's branching tree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier for this node within the scene.
    pub id: String,
    /// Localisation key for the node's narrative text.
    pub text_key: String,
    /// Available player choices from this node.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<Choice>,
}

/// A complete authored scene.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scene {
    /// Unique scene identifier.
    pub id: String,
    /// Which act of the story this scene belongs to.
    pub act: Act,
    /// Characters participating in this scene.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub participants: Vec<String>,
    /// Flag condition that must be satisfied for this scene to be available.
    #[serde(default)]
    pub requires: FlagExpr,
    /// The nodes that make up this scene's branching tree.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<Node>,
    /// Effects applied when leaving this scene.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_exit: Option<Effects>,
    /// Whether this scene is a terminal (ending) node.
    #[serde(default)]
    pub terminal: bool,
}
