use super::{ItemMut, List};
use core::fmt::Debug;
use std::any::type_name;
use std::collections::{
    hash_map::{Entry, VacantEntry},
    HashMap,
};
use std::ops::Deref;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct TxItemMut<'a, T>(TxItemMutInner<'a, T>);

#[derive(Debug)]
enum TxItemMutInner<'a, T> {
    Original(&'a T, VacantEntry<'a, usize, T>),
    ReplacedOrPushed(&'a mut T),
}

impl<'a, T> ItemMut<'a, T> for TxItemMut<'a, T> {
    fn set(&mut self, item: T) {
        ::replace_with::replace_with_or_abort(&mut self.0, |kind| match kind {
            TxItemMutInner::Original(_, vacant) => {
                TxItemMutInner::ReplacedOrPushed(vacant.insert(item))
            }
            TxItemMutInner::ReplacedOrPushed(item_mut_ref) => {
                *item_mut_ref = item;
                TxItemMutInner::ReplacedOrPushed(item_mut_ref)
            }
        });
    }
    fn get(&self) -> &T {
        match &self.0 {
            TxItemMutInner::Original(base_item_mut, _) => base_item_mut,
            TxItemMutInner::ReplacedOrPushed(item) => item,
        }
    }
}

pub struct TxList<L: Deref>
where
    L::Target: List,
{
    base: L,
    patch: TxListPatch<<<L as Deref>::Target as List>::Item>,
}

impl<L: Deref + Clone> Clone for TxList<L>
where
    L::Target: List,
    <L::Target as List>::Item: Clone,
{
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            patch: self.patch.clone(),
        }
    }
}
impl<L: Deref + Debug> Debug for TxList<L>
where
    L::Target: List,
    <L::Target as List>::Item: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("base", &self.base)
            .field("patch", &self.patch)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct TxListPatch<T> {
    replaced_items: HashMap<usize, T>,
    pushed_items: Vec<T>,
}
impl<T> TxListPatch<T> {
    pub fn apply<L: List<Item = T>>(self, list: &mut L) {
        for (key, item) in self.replaced_items {
            list.get_mut(key).unwrap().set(item)
        }
        for item in self.pushed_items {
            list.push(item)
        }
    }
}

impl<T> Default for TxListPatch<T> {
    fn default() -> Self {
        Self {
            replaced_items: HashMap::new(),
            pushed_items: Vec::new(),
        }
    }
}

impl<L: Deref> TxList<L>
where
    L::Target: List,
{
    pub fn new(base: L) -> Self {
        Self {
            base,
            patch: TxListPatch::default(),
        }
    }
    pub fn into_inner(self) -> (L, TxListPatch<<<L as Deref>::Target as List>::Item>) {
        (self.base, self.patch)
    }
}

impl<L: Deref> List for TxList<L>
where
    L::Target: List,
{
    type Item = <L::Target as List>::Item;
    type ItemMut<'a> = TxItemMut<'a, Self::Item> where Self: 'a;

    fn len(&self) -> usize {
        self.base.len() + self.patch.pushed_items.len()
    }
    fn push(&mut self, item: Self::Item) {
        self.patch.pushed_items.push(item);
    }
    fn get(&self, idx: usize) -> Option<&Self::Item> {
        if let Some(idx_in_pushed_items) = idx.checked_sub(self.base.len()) {
            self.patch.pushed_items.get(idx_in_pushed_items)
        } else if let Some(replaced_item) = self.patch.replaced_items.get(&idx) {
            Some(replaced_item)
        } else {
            self.base.get(idx)
        }
    }
    fn get_mut(&mut self, idx: usize) -> Option<Self::ItemMut<'_>> {
        Some(TxItemMut(
            if let Some(idx_in_pushed_items) = idx.checked_sub(self.base.len()) {
                let pushed_item = self
                    .patch
                    .pushed_items
                    .as_mut_slice()
                    .get_mut(idx_in_pushed_items)?;
                TxItemMutInner::ReplacedOrPushed(pushed_item)
            } else {
                match self.patch.replaced_items.entry(idx) {
                    Entry::Occupied(occupied) => {
                        TxItemMutInner::ReplacedOrPushed(occupied.into_mut())
                    }
                    Entry::Vacant(vacant) => {
                        let original_ref = self.base.get(idx)?;
                        TxItemMutInner::Original(original_ref, vacant)
                    }
                }
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list::tx::TxList;
    use alloc::rc::Rc;

    fn to_vec<L: List>(list: &L) -> Vec<L::Item>
    where
        L::Item: Clone,
    {
        let mut result = Vec::<L::Item>::new();

        let mut idx = 0;
        while let Some(item) = list.get(idx) {
            result.push(item.clone());
            idx += 1;
        }
        assert_eq!(result.len(), list.len());
        result
    }

    struct TextContext {
        base_values: Rc<Vec<u32>>,
        tx_list: TxList<Rc<Vec<u32>>>,
    }
    impl TextContext {
        fn new(base_values: &'static [u32]) -> Self {
            let base_values = Rc::new(base_values.to_vec());
            Self {
                base_values: Rc::clone(&base_values),
                tx_list: TxList::new(base_values),
            }
        }
        fn expect_values(&self, expected_values: &[u32]) {
            assert_eq!(to_vec(&self.tx_list), expected_values);
            let mut values = self.base_values.to_vec();
            self.tx_list.clone().into_inner().1.apply(&mut values);
            assert_eq!(values, expected_values);
        }
    }

    #[test]
    fn push() {
        let mut ctx = TextContext::new(&[4]);
        ctx.tx_list.push(2);
        ctx.expect_values(&[4, 2]);
    }

    #[test]
    fn entry_original() {
        let mut ctx = TextContext::new(&[2, 3]);
        let mut entry = ctx.tx_list.get_mut(0).unwrap();
        assert_eq!(*entry.get(), 2);
        entry.set(5);
        assert_eq!(*entry.get(), 5);
        ctx.expect_values(&[5, 3]);
    }

    #[test]
    fn entry_original_set_twice() {
        let mut ctx = TextContext::new(&[2, 3]);
        let mut entry = ctx.tx_list.get_mut(0).unwrap();
        entry.set(5);
        entry.set(8);
        assert_eq!(*entry.get(), 8);
        ctx.expect_values(&[8, 3]);
    }

    #[test]
    fn entry_pushed() {
        let mut ctx = TextContext::new(&[2, 3]);
        ctx.tx_list.push(6);
        let mut entry = ctx.tx_list.get_mut(2).unwrap();
        assert_eq!(*entry.get(), 6);
        entry.set(4);
        assert_eq!(*entry.get(), 4);
        ctx.expect_values(&[2, 3, 4]);
    }
    #[test]
    fn entry_pushed_set_twice() {
        let mut ctx = TextContext::new(&[2, 3]);
        ctx.tx_list.push(6);
        let mut entry = ctx.tx_list.get_mut(2).unwrap();
        entry.set(4);
        entry.set(7);
        assert_eq!(*entry.get(), 7);
        ctx.expect_values(&[2, 3, 7]);
    }
}
