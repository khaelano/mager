use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Demographic {
    Shounen,
    Shoujo,
    Josei,
    Seinen
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PublicationStatus {
    Ongoing,
    Completed,
    Hiatus,
    Cancelled
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ContentRating {
    Safe,
    Suggestive,
    Erotica,
    Pornographic
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    Manga,
    Chapter,
    CoverArt,
    Author,
    Artist,
    ScanlationGroup,
    Tag,
    User,
    CustomList,
    Creator
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Draft,
    Submitted,
    Published,
    Rejected
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MangaRelationshipType {
    Monochrome,
    MainStory,
    AdaptedFrom,
    BasedOn,
    Prequel,
    SideStory,
    Doujinshi,
    SameFranchise,
    SharedUniverse,
    Sequel,
    SpinOff,
    AlternateStory,
    AlternateVersion,
    Preserialization,
    Colored,
    Serialization
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TagGroup {
    Content,
    Format,
    Genre,
    Theme
}
