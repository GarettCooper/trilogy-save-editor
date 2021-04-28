use anyhow::Result;
use derive_more::{Deref, DerefMut, Display, From};
use imgui::ImString;
use indexmap::IndexMap;
use serde::de;
use std::{
    fmt::{self, Display},
    hash::Hash,
};

use crate::gui::Gui;

pub mod common;
pub mod mass_effect_1;
pub mod mass_effect_2;
pub mod mass_effect_3;

// Raw Ui
pub trait RawUi {
    fn draw_raw_ui(&mut self, gui: &Gui, ident: &str);
}

// Nouveau string type pour pouvoir implémenter serde...
#[derive(Deref, DerefMut, From, Clone, Default, PartialEq, Eq, Hash, Display)]
pub struct ImguiString(ImString);

impl RawUi for ImguiString {
    fn draw_raw_ui(&mut self, gui: &Gui, ident: &str) {
        gui.draw_edit_string(ident, &mut self.0);
    }
}

impl<'de> serde::Deserialize<'de> for ImguiString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self(ImString::new(string)))
    }
}

impl serde::Serialize for ImguiString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.to_str())
    }
}

// Implémentation des dummy
#[derive(Clone)]
pub struct Dummy<const LEN: usize>([u8; LEN]);

impl<const LEN: usize> Default for Dummy<LEN> {
    fn default() -> Self {
        Self([0; LEN])
    }
}

impl<'de, const LEN: usize> serde::Deserialize<'de> for Dummy<LEN> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DummyVisitor<const LEN: usize>;
        impl<'de, const LEN: usize> de::Visitor<'de> for DummyVisitor<LEN> {
            type Value = Dummy<LEN>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a seq")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut result = [0u8; LEN];
                let mut i = 0;
                while let Some(element) = seq.next_element()? {
                    result[i] = element;
                    i += 1;
                }
                Ok(Dummy(result))
            }
        }
        deserializer.deserialize_tuple_struct("Dummy<LEN>", LEN, DummyVisitor)
    }
}

impl<const LEN: usize> serde::Serialize for Dummy<LEN> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

// Implémentation des types std
impl RawUi for i32 {
    fn draw_raw_ui(&mut self, gui: &Gui, ident: &str) {
        gui.draw_edit_i32(ident, self);
    }
}

impl RawUi for f32 {
    fn draw_raw_ui(&mut self, gui: &Gui, ident: &str) {
        gui.draw_edit_f32(ident, self);
    }
}

impl RawUi for bool {
    fn draw_raw_ui(&mut self, gui: &Gui, ident: &str) {
        gui.draw_edit_bool(ident, self);
    }
}

impl<T> RawUi for Vec<T>
where
    T: RawUi + Default,
{
    fn draw_raw_ui(&mut self, gui: &Gui, ident: &str) {
        gui.draw_vec(ident, self);
    }
}

impl<K, V> RawUi for IndexMap<K, V>
where
    K: RawUi + Eq + Hash + Default + Display,
    V: RawUi + Default,
{
    fn draw_raw_ui(&mut self, gui: &Gui, ident: &str) {
        gui.draw_indexmap(ident, self);
    }
}
